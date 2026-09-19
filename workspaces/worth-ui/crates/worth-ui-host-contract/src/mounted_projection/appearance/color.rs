#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedAppearanceColor([u8; 4]);

impl UiMountedAppearanceColor {
    pub const fn from_straight_srgba(channels: [u8; 4]) -> Self {
        Self(channels)
    }
    pub const fn straight_srgba(self) -> [u8; 4] {
        self.0
    }
}
