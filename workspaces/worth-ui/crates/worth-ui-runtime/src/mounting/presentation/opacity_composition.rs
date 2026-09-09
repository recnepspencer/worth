use worth_ui_host_contract::{UiMountedAppearanceOpacity, UiMountedPresentationOpacity};

/// Consume the two raw factors once. Final presentation opacity is deliberately
/// not an input type, so retained composed output cannot be multiplied again.
pub(in crate::mounting) fn compose_opacity(
    appearance: UiMountedAppearanceOpacity,
    motion_units: u16,
) -> UiMountedPresentationOpacity {
    let product = u32::from(appearance.units()) * u32::from(motion_units);
    let denominator = u32::from(u16::MAX);
    let quotient = product / denominator;
    let remainder = product % denominator;
    let round_up =
        remainder * 2 > denominator || (remainder * 2 == denominator && quotient % 2 == 1);
    let units = (quotient + u32::from(round_up)).min(denominator) as u16;
    UiMountedPresentationOpacity::from_runtime_composition(units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_factor_preserves_exact_zero_and_identity() {
        for units in 0..=u16::MAX {
            let appearance = UiMountedAppearanceOpacity::from_units(units);
            assert_eq!(compose_opacity(appearance, 0).units(), 0);
            assert_eq!(compose_opacity(appearance, u16::MAX).units(), units);
            assert_eq!(
                compose_opacity(UiMountedAppearanceOpacity::ZERO, units).units(),
                0
            );
            assert_eq!(
                compose_opacity(UiMountedAppearanceOpacity::ONE, units).units(),
                units
            );
        }
    }

    #[test]
    fn composition_preserves_non_u8_factors_and_rounds_to_the_nearest_unit() {
        // A distance oracle compares the adjacent representable values instead
        // of repeating the producer's remainder/rounding branch.
        for appearance in [
            1, 127, 255, 256, 1_023, 10_000, 12_345, 32_768, 40_000, 65_534,
        ] {
            for motion in [1, 128, 257, 1_111, 8_192, 23_417, 32_768, 58_367, 65_534] {
                let exact = u64::from(appearance) * u64::from(motion);
                let lower = exact / 65_535;
                let upper = lower + 1;
                let expected = [lower, upper]
                    .into_iter()
                    .min_by_key(|candidate| ((candidate * 65_535).abs_diff(exact), candidate % 2))
                    .unwrap();
                assert_eq!(
                    u64::from(
                        compose_opacity(UiMountedAppearanceOpacity::from_units(appearance), motion)
                            .units()
                    ),
                    expected,
                );
            }
        }
        assert_eq!(
            compose_opacity(UiMountedAppearanceOpacity::from_units(40_000), 32_768).units(),
            20_000
        );
        assert_eq!(
            compose_opacity(UiMountedAppearanceOpacity::from_units(32_768), 32_768).units(),
            16_384
        );
        // The odd denominator makes an exact half-unit tie unreachable for
        // integer factors, but the producer still states the ties-even rule.
    }
}
