#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedRgba8([u8; 4]);

impl UiMountedRgba8 {
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([red, green, blue, alpha])
    }

    pub const fn channels(self) -> [u8; 4] {
        self.0
    }
}
