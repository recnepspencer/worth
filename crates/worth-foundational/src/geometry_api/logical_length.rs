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

    /// `points` whole points. `None` past every length [`Self::new`] admits.
    pub const fn whole_points(points: u32) -> Option<Self> {
        match points.checked_mul(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT) {
            Some(subpixels) if subpixels <= i32::MAX.unsigned_abs() => Some(Self(subpixels)),
            _ => None,
        }
    }

    /// This length in points.
    pub fn points_f32(self) -> f32 {
        super::logical_points::appearance_points_f32(i64::from(self.0))
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

    #[test]
    fn whole_points_count_a_thousand_subpixels_each_up_to_the_largest_length() {
        assert_eq!(
            UiAppearanceLogicalLength::whole_points(3)
                .unwrap()
                .subpixels(),
            3_000
        );
        assert_eq!(
            UiAppearanceLogicalLength::whole_points(3)
                .unwrap()
                .points_f32(),
            3.0
        );
        assert!(UiAppearanceLogicalLength::whole_points(2_147_483).is_some());
        assert_eq!(UiAppearanceLogicalLength::whole_points(2_147_484), None);
        assert_eq!(UiAppearanceLogicalLength::whole_points(u32::MAX), None);
    }
}
