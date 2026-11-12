//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::candidate::inline_type_budget::collect(census);
    super::denial::inline_type_budget::collect(census);
    super::reservation::inline_type_budget::collect(census);
    super::step::inline_type_budget::collect(census);
}
