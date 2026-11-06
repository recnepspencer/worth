//! Image admission preserves the same compared prefix and correspondence.
use super::{
    comparison::{self, HostDivergence, HostOracleComparison},
    projection::StateProjection,
};
/// Both sides of an image cut preserve the independently executed prefix and
/// revisit the SAME map. No image header, coverage or recovery proof is invented.
pub fn compare_image_restart(
    comparison: &mut HostOracleComparison,
    before: &StateProjection,
    reopened: &StateProjection,
    reference: &StateProjection,
) -> Result<(), HostDivergence> {
    comparison::compare(comparison, before, reference)?;
    comparison::compare(comparison, reopened, reference)
}
