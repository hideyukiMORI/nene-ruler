//! 窓に使う visual。透明には depth 32 が要る（ADR 0005 の 1）。

use x11rb::protocol::xproto::{Screen, VisualClass, Visualid};

/// depth と visual と、それに合う cairo の画素形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowVisual {
    depth: u8,
    visual: Visualid,
    format: cairo::Format,
}

impl WindowVisual {
    /// depth 32 の `TrueColor` visual。無ければ `None`。
    #[must_use]
    pub fn argb(screen: &Screen) -> Option<Self> {
        screen
            .allowed_depths
            .iter()
            .filter(|depth| depth.depth == 32)
            .flat_map(|depth| depth.visuals.iter())
            .find(|visual| visual.class == VisualClass::TRUE_COLOR)
            .map(|visual| Self {
                depth: 32,
                visual: visual.visual_id,
                format: cairo::Format::ARgb32,
            })
    }

    /// 根の窓と同じ visual。depth 24 のときだけ cairo の `Rgb24` と並びが合う。
    #[must_use]
    pub fn root(screen: &Screen) -> Option<Self> {
        (screen.root_depth == 24).then_some(Self {
            depth: 24,
            visual: screen.root_visual,
            format: cairo::Format::Rgb24,
        })
    }

    /// depth。
    #[must_use]
    pub const fn depth(self) -> u8 {
        self.depth
    }

    /// visual の ID。
    #[must_use]
    pub const fn visual(self) -> Visualid {
        self.visual
    }

    /// cairo の画素形式。
    #[must_use]
    pub const fn format(self) -> cairo::Format {
        self.format
    }
}
