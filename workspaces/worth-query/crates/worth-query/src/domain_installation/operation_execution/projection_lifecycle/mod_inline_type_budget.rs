//! Existing lane and phase layouts; no witness is minted.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::operational_owner::inline_type_budget::collect(census);
    super::promotion_outcome::inline_type_budget::collect(census);
    super::states::inline_type_budget::collect(census);
}
