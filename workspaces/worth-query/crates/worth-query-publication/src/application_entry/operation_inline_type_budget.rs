//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::close::inline_type_budget::collect(census);
    super::source_only::inline_type_budget::collect(census);
    super::workflow::inline_type_budget::collect(census);
    super::fixed::inline_type_budget::collect(census);
}
