//! Mounted placement carried into glyph-demand geometry and subpixel keys.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiGlyphRasterPlacement {
    origin_x_millipoints: i64,
    origin_y_millipoints: i64,
}

impl UiGlyphRasterPlacement {
    /// A placement already in millipoints, such as a mounted text command's
    /// presented origin.
    pub const fn from_millipoints(origin: [i64; 2]) -> Self {
        Self {
            origin_x_millipoints: origin[0],
            origin_y_millipoints: origin[1],
        }
    }

    pub const fn origin_x_millipoints(self) -> i64 {
        self.origin_x_millipoints
    }

    pub const fn origin_y_millipoints(self) -> i64 {
        self.origin_y_millipoints
    }
}
