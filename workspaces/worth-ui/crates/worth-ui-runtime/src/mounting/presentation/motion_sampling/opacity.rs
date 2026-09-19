/// Quantize at sampling, before retaining a value for publication or retargeting.
/// The sampler supplies finite, eased progress in the closed unit interval.
pub(super) fn interpolate_units(start: u16, end: u16, progress: f32) -> u16 {
    let start = f64::from(start);
    let sampled = start + (f64::from(end) - start) * f64::from(progress);
    sampled.round_ties_even().clamp(0.0, f64::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::interpolate_units;

    #[test]
    fn sampling_rounds_both_tie_parities_in_unit_space() {
        assert_eq!(interpolate_units(0, 1, 0.5), 0);
        assert_eq!(interpolate_units(1, 2, 0.5), 2);
        assert_eq!(interpolate_units(2, 1, 0.5), 2);
        assert_eq!(interpolate_units(1, 0, 0.5), 0);
        assert_eq!(interpolate_units(0, u16::MAX, 0.5), 32_768);
        assert_eq!(interpolate_units(0, u16::MAX, 0.25), 16_384);
    }

    #[test]
    fn sampling_preserves_every_canonical_endpoint() {
        for units in 0..=u16::MAX {
            assert_eq!(interpolate_units(units, 0, 0.0), units);
            assert_eq!(interpolate_units(u16::MAX, units, 1.0), units);
            assert_eq!(interpolate_units(units, units, 0.375), units);
        }
    }
}
