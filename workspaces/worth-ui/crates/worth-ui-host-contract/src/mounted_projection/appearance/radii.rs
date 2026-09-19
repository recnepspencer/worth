#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceNormalizedLogicalRadii {
    corners: [u32; 4],
    basis_width: u32,
    basis_height: u32,
}

impl UiAppearanceNormalizedLogicalRadii {
    pub fn normalize(
        bounds: super::UiAppearanceAllocationBounds,
        authored: [super::UiAppearanceLogicalLength; 4],
    ) -> Self {
        let authored = authored.map(super::UiAppearanceLogicalLength::subpixels);
        let [top_left, top_right, bottom_right, bottom_left] = authored;
        let constraints = [
            (
                u64::from(bounds.width()),
                u64::from(top_left) + u64::from(top_right),
            ),
            (
                u64::from(bounds.width()),
                u64::from(bottom_left) + u64::from(bottom_right),
            ),
            (
                u64::from(bounds.height()),
                u64::from(top_left) + u64::from(bottom_left),
            ),
            (
                u64::from(bounds.height()),
                u64::from(top_right) + u64::from(bottom_right),
            ),
        ];
        let (numerator, denominator) = constraints.into_iter().filter(|(_, sum)| *sum != 0).fold(
            (1_u64, 1_u64),
            |best, candidate| {
                if u128::from(candidate.0) * u128::from(best.1)
                    < u128::from(best.0) * u128::from(candidate.1)
                {
                    candidate
                } else {
                    best
                }
            },
        );
        if numerator >= denominator {
            return Self {
                corners: authored,
                basis_width: bounds.width(),
                basis_height: bounds.height(),
            };
        }
        let rounded =
            authored.map(|radius| round_scaled_to_nearest_even(radius, numerator, denominator));
        let top_left = rounded[0];
        let top_right = rounded[1].min(bounds.width().saturating_sub(top_left));
        let bottom_right = rounded[2].min(bounds.height().saturating_sub(top_right));
        let bottom_left = rounded[3]
            .min(bounds.width().saturating_sub(bottom_right))
            .min(bounds.height().saturating_sub(top_left));
        Self {
            corners: [top_left, top_right, bottom_right, bottom_left],
            basis_width: bounds.width(),
            basis_height: bounds.height(),
        }
    }

    pub const fn corners(self) -> [u32; 4] {
        self.corners
    }

    pub(crate) const fn matches_allocation(
        self,
        allocation: super::UiAppearanceAllocationBounds,
    ) -> bool {
        self.basis_width == allocation.width() && self.basis_height == allocation.height()
    }

    pub(crate) fn with_outline_offset(
        self,
        offset: super::UiAppearanceLogicalLength,
    ) -> Result<Self, super::UiAppearanceGeometryOverflow> {
        let doubled_offset = offset
            .subpixels()
            .checked_mul(2)
            .ok_or(super::UiAppearanceGeometryOverflow)?;
        let basis_width = self
            .basis_width
            .checked_add(doubled_offset)
            .ok_or(super::UiAppearanceGeometryOverflow)?;
        let basis_height = self
            .basis_height
            .checked_add(doubled_offset)
            .ok_or(super::UiAppearanceGeometryOverflow)?;
        let mut radii = [0; 4];
        for (output, radius) in radii.iter_mut().zip(self.corners) {
            *output = radius
                .checked_add(offset.subpixels())
                .ok_or(super::UiAppearanceGeometryOverflow)?;
        }
        Ok(Self {
            corners: radii,
            basis_width,
            basis_height,
        })
    }
}

fn round_scaled_to_nearest_even(value: u32, numerator: u64, denominator: u64) -> u32 {
    let product = u128::from(value) * u128::from(numerator);
    let denominator = u128::from(denominator);
    let quotient = product / denominator;
    let remainder = product % denominator;
    let twice_remainder = remainder * 2;
    let increment =
        twice_remainder > denominator || (twice_remainder == denominator && quotient % 2 == 1);
    (quotient + u128::from(increment)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn length(value: i32) -> super::super::UiAppearanceLogicalLength {
        super::super::UiAppearanceLogicalLength::new(value).unwrap()
    }

    #[test]
    fn radii_normalize_once_in_logical_subpixels_without_overflow() {
        let bounds = super::super::UiAppearanceAllocationBounds::new(0, 0, 100, 50).unwrap();
        assert_eq!(
            UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [length(80), length(40), length(40), length(80)]
            )
            .corners(),
            [25, 12, 12, 25]
        );
        let huge =
            super::super::UiAppearanceAllocationBounds::new(0, 0, u32::MAX, u32::MAX).unwrap();
        assert_eq!(
            UiAppearanceNormalizedLogicalRadii::normalize(huge, [length(i32::MAX); 4]).corners(),
            [i32::MAX as u32; 4]
        );
    }

    #[test]
    fn logical_radius_half_ties_are_even_and_never_cross_an_edge() {
        let bounds = super::super::UiAppearanceAllocationBounds::new(0, 0, 3, 100).unwrap();
        let normalized = UiAppearanceNormalizedLogicalRadii::normalize(
            bounds,
            [length(3), length(3), length(0), length(0)],
        );
        assert_eq!(normalized.corners(), [2, 1, 0, 0]);
        assert_eq!(round_scaled_to_nearest_even(5, 1, 2), 2);
        assert_eq!(round_scaled_to_nearest_even(7, 1, 2), 4);
    }

    #[test]
    fn outline_offset_carries_checked_expanded_normalization_dimensions() {
        let original = super::super::UiAppearanceAllocationBounds::new(0, 0, 100, 100).unwrap();
        let shifted = UiAppearanceNormalizedLogicalRadii::normalize(original, [length(50); 4])
            .with_outline_offset(length(1))
            .unwrap();
        let expanded = super::super::UiAppearanceAllocationBounds::new(-1, -1, 102, 102).unwrap();
        assert_eq!(shifted.corners(), [51; 4]);
        assert!(!shifted.matches_allocation(original));
        assert!(shifted.matches_allocation(expanded));

        let maximum =
            super::super::UiAppearanceAllocationBounds::new(0, 0, u32::MAX, u32::MAX).unwrap();
        let maximum_radii = UiAppearanceNormalizedLogicalRadii::normalize(
            maximum,
            [super::super::UiAppearanceLogicalLength::ZERO; 4],
        );
        assert_eq!(
            maximum_radii.with_outline_offset(length(1)),
            Err(super::super::UiAppearanceGeometryOverflow)
        );
    }
}
