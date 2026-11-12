//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::contract::inline_type_budget::collect(census);
    super::execution::inline_type_budget::collect(census);
}
