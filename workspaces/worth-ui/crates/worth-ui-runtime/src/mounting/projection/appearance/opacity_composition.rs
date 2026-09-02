use worth_ui_host_contract::UiMountedAppearanceOpacity;

pub(super) fn compose(
    appearance: UiMountedAppearanceOpacity,
    motion: Option<UiMountedAppearanceOpacity>,
) -> UiMountedAppearanceOpacity {
    appearance.compose(motion.unwrap_or(UiMountedAppearanceOpacity::ONE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_motion_is_the_exact_identity() {
        let appearance = UiMountedAppearanceOpacity::from_units(40_000);
        assert_eq!(compose(appearance, None), appearance);
    }

    #[test]
    fn composition_keeps_u16_precision_until_the_single_product() {
        let appearance = UiMountedAppearanceOpacity::from_units(40_000);
        let motion = UiMountedAppearanceOpacity::from_units(32_768);
        let product = u128::from(appearance.units()) * u128::from(motion.units());
        let denominator = u128::from(u16::MAX);
        let quotient = product / denominator;
        let remainder = product % denominator;
        let expected = quotient + u128::from(remainder * 2 >= denominator);
        assert_eq!(
            compose(appearance, Some(motion)).units(),
            u16::try_from(expected).unwrap()
        );
        assert_eq!(compose(appearance, Some(motion)).units(), 20_000);
    }

    #[test]
    fn zero_and_one_are_exact_composition_identities() {
        let motion = UiMountedAppearanceOpacity::from_units(12_345);
        assert_eq!(
            compose(UiMountedAppearanceOpacity::ZERO, Some(motion)),
            UiMountedAppearanceOpacity::ZERO
        );
        assert_eq!(
            compose(UiMountedAppearanceOpacity::ONE, Some(motion)),
            motion
        );
    }
}
