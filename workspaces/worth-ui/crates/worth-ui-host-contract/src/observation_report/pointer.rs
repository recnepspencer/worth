#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiHostPointerIdentity(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiHostPointerCaptureEpoch(u64);

pub const UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT: i64 = 1_000;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UiHostPointerDeviceKind {
    Mouse,
    Stylus,
    Touch,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UiHostSurfaceCoordinateSpace {
    Viewport,
    Window,
    HostSurface,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UiHostSurfaceCoordinateUnit {
    LogicalPoint,
    PhysicalPixel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiHostSurfacePositionBasis {
    coordinate_space: UiHostSurfaceCoordinateSpace,
    coordinate_unit: UiHostSurfaceCoordinateUnit,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiHostSurfacePosition {
    basis: UiHostSurfacePositionBasis,
    x_subpixels: i64,
    y_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiHostPressedPointerButtons(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UiHostPointerButton {
    Primary,
    Secondary,
    Middle,
    Extra1,
    Extra2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UiHostPointerButtonTransition {
    Pressed,
    Released,
}

impl UiHostPointerIdentity {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl UiHostPointerCaptureEpoch {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl UiHostSurfacePositionBasis {
    pub const fn new(
        coordinate_space: UiHostSurfaceCoordinateSpace,
        coordinate_unit: UiHostSurfaceCoordinateUnit,
    ) -> Self {
        Self {
            coordinate_space,
            coordinate_unit,
        }
    }

    pub const fn viewport_logical() -> Self {
        Self::new(
            UiHostSurfaceCoordinateSpace::Viewport,
            UiHostSurfaceCoordinateUnit::LogicalPoint,
        )
    }

    pub const fn coordinate_space(self) -> UiHostSurfaceCoordinateSpace {
        self.coordinate_space
    }

    pub const fn coordinate_unit(self) -> UiHostSurfaceCoordinateUnit {
        self.coordinate_unit
    }
}

impl UiHostSurfacePosition {
    pub const fn new(
        basis: UiHostSurfacePositionBasis,
        x_subpixels: i64,
        y_subpixels: i64,
    ) -> Self {
        Self {
            basis,
            x_subpixels,
            y_subpixels,
        }
    }

    pub const fn viewport_logical(x_subpixels: i64, y_subpixels: i64) -> Self {
        Self::new(
            UiHostSurfacePositionBasis::viewport_logical(),
            x_subpixels,
            y_subpixels,
        )
    }

    pub const fn basis(self) -> UiHostSurfacePositionBasis {
        self.basis
    }

    pub const fn x_subpixels(self) -> i64 {
        self.x_subpixels
    }

    pub const fn y_subpixels(self) -> i64 {
        self.y_subpixels
    }
}

impl UiHostPressedPointerButtons {
    pub const NONE: Self = Self(0);

    pub fn from_buttons(buttons: impl IntoIterator<Item = UiHostPointerButton>) -> Self {
        let mut bits = 0u8;
        for button in buttons {
            bits |= button.bit();
        }
        Self(bits)
    }

    pub const fn contains(self, button: UiHostPointerButton) -> bool {
        self.0 & button.bit() != 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl UiHostPointerButton {
    const fn bit(self) -> u8 {
        1 << self as u8
    }
}
