//! CLIPBOARD の所有と待ち受け（FR-007 / ADR 0007）。

use std::sync::{Arc, Mutex};
use std::thread;

use nene_ruler_application::clipboard_failure::ClipboardFailure;
use nene_ruler_application::clipboard_port::ClipboardPort;
use x11rb::connection::Connection as _;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, SELECTION_NOTIFY_EVENT,
    SelectionNotifyEvent, SelectionRequestEvent, Window, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::adapter_failure::AdapterFailure;
use crate::selection_answer::{SelectionAnswer, reply_property};
use crate::selection_atoms::SelectionAtoms;

/// 専用の接続・不可視の窓・待ち受けのスレッド。置いた文字列は `Mutex` で共有する。
#[derive(Debug)]
pub struct X11Clipboard {
    connection: Arc<RustConnection>,
    window: Window,
    atoms: SelectionAtoms,
    text: Arc<Mutex<String>>,
}

impl X11Clipboard {
    /// `DISPLAY` に繋ぎ、所有者になる窓を作り、待ち受けを始める。
    ///
    /// # Errors
    ///
    /// 繋げない・画面が無い・窓が作れない。
    pub fn connect() -> Result<Self, AdapterFailure> {
        let (connection, screen_index) = x11rb::connect(None)?;
        let root = connection
            .setup()
            .roots
            .get(screen_index)
            .map(|screen| screen.root)
            .ok_or(AdapterFailure::Connect(
                x11rb::errors::ConnectError::InvalidScreen,
            ))?;
        let atoms = SelectionAtoms::intern(&connection)?;
        let window = connection.generate_id()?;
        connection.create_window(
            0,
            window,
            root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_ONLY,
            0,
            &CreateWindowAux::new().event_mask(EventMask::NO_EVENT),
        )?;
        connection.flush()?;
        let connection = Arc::new(connection);
        let text = Arc::new(Mutex::new(String::new()));
        let served = Arc::clone(&connection);
        let shared = Arc::clone(&text);
        thread::spawn(move || serve(&served, atoms, &shared));
        Ok(Self {
            connection,
            window,
            atoms,
            text,
        })
    }
    /// 所有を取り、X が誰を所有者と見ているかを返す。
    fn own(&self) -> Result<Window, AdapterFailure> {
        self.connection
            .set_selection_owner(self.window, self.atoms.clipboard, x11rb::CURRENT_TIME)?
            .check()?;
        Ok(self
            .connection
            .get_selection_owner(self.atoms.clipboard)?
            .reply()?
            .owner)
    }
}

impl ClipboardPort for X11Clipboard {
    fn copy(&self, text: &str) -> Result<(), ClipboardFailure> {
        {
            let mut held = self
                .text
                .lock()
                .map_err(|_poisoned| ClipboardFailure::Unavailable)?;
            held.clear();
            held.push_str(text);
        }
        let owned = self.own().map_err(|_error| ClipboardFailure::Unavailable)?;
        if owned == self.window {
            Ok(())
        } else {
            Err(ClipboardFailure::Rejected)
        }
    }
}

/// 待ち受け。`SelectionRequest` に答え続け、接続が切れたら終わる。
fn serve(connection: &RustConnection, atoms: SelectionAtoms, text: &Mutex<String>) {
    loop {
        let Ok(event) = connection.wait_for_event() else {
            return;
        };
        if let Event::SelectionRequest(request) = event {
            // 1 件の答えの失敗（要求者が消えた等）で待ち受けを止めない。
            let _outcome = answer(connection, atoms, text, &request);
        }
    }
}

/// 1 件の要求に答え、`SelectionNotify` を送る。
fn answer(
    connection: &RustConnection,
    atoms: SelectionAtoms,
    text: &Mutex<String>,
    request: &SelectionRequestEvent,
) -> Result<(), AdapterFailure> {
    let property = reply_property(request.property, request.target);
    let written = match SelectionAnswer::decide(request.target, atoms) {
        SelectionAnswer::Targets => {
            connection.change_property32(
                x11rb::protocol::xproto::PropMode::REPLACE,
                request.requestor,
                property,
                AtomEnum::ATOM,
                &SelectionAnswer::targets(atoms),
            )?;
            property
        }
        SelectionAnswer::Text(kind) => {
            let held = text.lock().map(|held| held.clone()).unwrap_or_default();
            connection.change_property8(
                x11rb::protocol::xproto::PropMode::REPLACE,
                request.requestor,
                property,
                kind,
                held.as_bytes(),
            )?;
            property
        }
        SelectionAnswer::Refuse => AtomEnum::NONE.into(),
    };
    let notify = SelectionNotifyEvent {
        response_type: SELECTION_NOTIFY_EVENT,
        sequence: 0,
        time: request.time,
        requestor: request.requestor,
        selection: request.selection,
        target: request.target,
        property: written,
    };
    connection.send_event(false, request.requestor, EventMask::NO_EVENT, notify)?;
    connection.flush()?;
    Ok(())
}
