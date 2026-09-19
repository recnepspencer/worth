#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedAppearanceOpacity(u16);

impl UiMountedAppearanceOpacity {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u16::MAX);

    pub const fn from_units(units: u16) -> Self {
        Self(units)
    }
    pub const fn units(self) -> u16 {
        self.0
    }
}
