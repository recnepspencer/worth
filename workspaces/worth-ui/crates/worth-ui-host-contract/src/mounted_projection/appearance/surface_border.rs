/// Allocation side containing one omitted interval of an inward surface border.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiMountedSurfaceBorderSide {
    Top,
    Right,
    Bottom,
    Left,
}

/// Half-open logical interval omitted from one allocation-side border.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedSurfaceBorderOmission {
    side: UiMountedSurfaceBorderSide,
    start: u32,
    end: u32,
}

impl UiMountedSurfaceBorderOmission {
    #[doc(hidden)]
    pub const fn from_runtime_mosaic(
        side: UiMountedSurfaceBorderSide,
        start: u32,
        end: u32,
    ) -> Option<Self> {
        if start < end {
            Some(Self { side, start, end })
        } else {
            None
        }
    }

    pub const fn side(self) -> UiMountedSurfaceBorderSide {
        self.side
    }

    pub const fn start(self) -> u32 {
        self.start
    }

    pub const fn end(self) -> u32 {
        self.end
    }
}
