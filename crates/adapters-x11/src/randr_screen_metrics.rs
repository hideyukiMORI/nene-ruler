//! `RandR` から画面の物理寸法を読む（FR-008 / FR-016 の元）。

use nene_ruler_application::screen_metrics_port::ScreenMetricsPort;
use nene_ruler_core::screen_metrics::ScreenMetrics;
use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;
use x11rb::connection::Connection as _;
use x11rb::errors::{ConnectError, ReplyError};

use crate::adapter_failure::AdapterFailure;
use x11rb::protocol::randr::{self, ConnectionExt as _, GetOutputInfoReply};
use x11rb::protocol::xproto::Window;
use x11rb::rust_connection::RustConnection;

/// 主出力（無ければ最初に繋がっている出力）の画素数とミリ寸法を、ポートを通して返す。
#[derive(Debug)]
pub struct RandrScreenMetrics {
    connection: RustConnection,
    root: Window,
}

impl RandrScreenMetrics {
    /// `DISPLAY` に繋ぐ。
    ///
    /// # Errors
    ///
    /// X サーバに繋げない、または既定の画面が無い。
    pub fn connect() -> Result<Self, AdapterFailure> {
        let (connection, screen_index) = x11rb::connect(None)?;
        let root = connection
            .setup()
            .roots
            .get(screen_index)
            .map(|screen| screen.root)
            .ok_or(ConnectError::InvalidScreen)?;
        Ok(Self { connection, root })
    }

    /// 主出力を優先し、無ければ繋がっていて CRTC を持つ最初の出力を選ぶ。
    fn read(&self) -> Result<Result<ScreenMetrics, ScreenMetricsFailure>, ReplyError> {
        let resources = self
            .connection
            .randr_get_screen_resources_current(self.root)?
            .reply()?;
        let primary = self
            .connection
            .randr_get_output_primary(self.root)?
            .reply()?
            .output;
        let candidates = core::iter::once(primary).chain(resources.outputs.iter().copied());
        for output in candidates.filter(|output| *output != 0) {
            let info = self
                .connection
                .randr_get_output_info(output, resources.config_timestamp)?
                .reply()?;
            if let Some(metrics) = self.metrics_of(&info, resources.config_timestamp)? {
                return Ok(metrics);
            }
        }
        Ok(Err(ScreenMetricsFailure::Unavailable))
    }

    /// 出力が繋がっていて CRTC を持つなら、その画素数と出力のミリ寸法を組にする。
    fn metrics_of(
        &self,
        info: &GetOutputInfoReply,
        config_timestamp: u32,
    ) -> Result<Option<Result<ScreenMetrics, ScreenMetricsFailure>>, ReplyError> {
        if info.connection != randr::Connection::CONNECTED || info.crtc == 0 {
            return Ok(None);
        }
        let crtc = self
            .connection
            .randr_get_crtc_info(info.crtc, config_timestamp)?
            .reply()?;
        Ok(Some(ScreenMetrics::new(
            u32::from(crtc.width),
            u32::from(crtc.height),
            info.mm_width,
            info.mm_height,
        )))
    }
}

impl ScreenMetricsPort for RandrScreenMetrics {
    fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
        // X の要求の失敗（RandR が無い等）は「取れなかった」の 1 つの閉じた理由に畳む（ARC-010）。
        self.read()
            .unwrap_or(Err(ScreenMetricsFailure::Unavailable))
    }
}
