pub const UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT: u32 = 1_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceLogicalLength(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceNegativeLength;

impl UiAppearanceLogicalLength {
    pub const ZERO: Self = Self(0);

    pub const fn new(subpixels: i32) -> Result<Self, UiAppearanceNegativeLength> {
        if subpixels < 0 {
            Err(UiAppearanceNegativeLength)
        } else {
            Ok(Self(subpixels as u32))
        }
    }

    pub const fn subpixels(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_length_uses_one_thousand_subpixels_and_denies_negative_values() {
        assert_eq!(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT, 1_000);
        assert_eq!(
            UiAppearanceLogicalLength::new(-1),
            Err(UiAppearanceNegativeLength)
        );
        assert_eq!(
            UiAppearanceLogicalLength::new(2_500).unwrap().subpixels(),
            2_500
        );
    }
}
