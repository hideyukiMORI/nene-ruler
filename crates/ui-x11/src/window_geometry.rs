//! 窓の根からの位置と大きさ。自前の移動・伸縮の計算に使う。

/// X の `ConfigureWindow` に渡す形（`i32` の位置と `u32` の大きさ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl WindowGeometry {
    /// 位置と大きさから作る。
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 根からの横位置。
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// 根からの縦位置。
    #[must_use]
    pub const fn y(self) -> i32 {
        self.y
    }

    /// 幅。
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}
