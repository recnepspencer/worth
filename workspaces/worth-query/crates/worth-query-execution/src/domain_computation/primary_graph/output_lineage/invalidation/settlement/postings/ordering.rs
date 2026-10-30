//! In-place fingerprint/exact-key/ordinal order; hashes never establish equality.
use super::ProjectedKey;
#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
pub(super) fn sort(projected: &mut [ProjectedKey]) {
    projected.sort_unstable_by(|left, right| {
        left.hash
            .cmp(&right.hash)
            .then_with(|| left.key.cmp(&right.key))
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
}
