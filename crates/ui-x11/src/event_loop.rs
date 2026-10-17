//! 枠の窓と設定モーダルの 2 窓にイベントを配る。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::Window;

use crate::display::Display;
use crate::flow::Flow;
use crate::ruler_window::RulerWindow;
use crate::settings_flow::SettingsFlow;
use crate::settings_window::SettingsWindow;
use crate::ui_failure::UiFailure;

/// 枠の窓を開き、イベントを待ち、意図を `controller` に渡し、描き直す。閉じる要求で戻る。
///
/// # Errors
///
/// X の要求か cairo が失敗した。
pub fn run(display: &Display, controller: &mut RulerController<'_>) -> Result<(), UiFailure> {
    let mut ruler = RulerWindow::open(display, &controller.frame())?;
    let mut settings: Option<SettingsWindow<'_>> = None;
    loop {
        let event = display.connection().wait_for_event()?;
        let for_settings = settings
            .as_ref()
            .is_some_and(|window| window_of(&event) == Some(window.window()));
        if for_settings {
            if let Some(window) = &settings
                && settings_step(controller, &ruler, window, &event)? == SettingsFlow::Close
            {
                controller.apply(RulerIntent::CloseSettings);
                // `Drop` が送る DestroyWindow は、次の要求まで送信バッファに残る。ここで流す。
                settings = None;
                display.connection().flush()?;
            }
        } else {
            match ruler.step(controller, &event)? {
                Flow::Continue => {}
                Flow::Quit => return Ok(()),
                Flow::OpenSettings => {
                    settings = open_settings(display, controller, &ruler, settings)?;
                }
            }
        }
    }
}

/// モーダル宛のイベント。設定が変わっていれば枠とモーダルを描き直す。
fn settings_step(
    controller: &mut RulerController<'_>,
    ruler: &RulerWindow<'_>,
    window: &SettingsWindow<'_>,
    event: &Event,
) -> Result<SettingsFlow, UiFailure> {
    let flow = window.handle(controller, event)?;
    if flow == SettingsFlow::Changed {
        ruler.refresh(controller)?;
        if let Some(frame) = controller.settings_frame() {
            window.present(&frame)?;
        }
    }
    Ok(flow)
}

/// ギアが押された。まだ開いていなければモーダルを開く。
fn open_settings<'d>(
    display: &'d Display,
    controller: &mut RulerController<'_>,
    ruler: &RulerWindow<'d>,
    current: Option<SettingsWindow<'d>>,
) -> Result<Option<SettingsWindow<'d>>, UiFailure> {
    if current.is_some() {
        return Ok(current);
    }
    controller.apply(RulerIntent::OpenSettings);
    let Some(frame) = controller.settings_frame() else {
        return Ok(None);
    };
    Ok(Some(SettingsWindow::open(display, ruler.window(), &frame)?))
}

/// イベントの宛先の窓。X のイベントは開いた集合なので `if let` の連なりで書く（規約 第 4 節）。
fn window_of(event: &Event) -> Option<Window> {
    if let Event::Expose(expose) = event {
        Some(expose.window)
    } else if let Event::ConfigureNotify(configure) = event {
        Some(configure.window)
    } else if let Event::ButtonPress(press) = event {
        Some(press.event)
    } else if let Event::ButtonRelease(release) = event {
        Some(release.event)
    } else if let Event::MotionNotify(motion) = event {
        Some(motion.event)
    } else if let Event::KeyPress(key) = event {
        Some(key.event)
    } else if let Event::ClientMessage(message) = event {
        Some(message.window)
    } else {
        None
    }
}
