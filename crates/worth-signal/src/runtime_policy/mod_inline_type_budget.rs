//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::conditional_evaluation::inline_type_budget::collect(census);
    super::conditional_temporal::inline_type_budget::collect(census);
    super::definition::inline_type_budget::collect(census);
}
