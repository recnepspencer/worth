//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::live::inline_type_budget::collect(census);
    super::live_read_result::inline_type_budget::collect(census);
    super::read_composition::inline_type_budget::collect(census);
    super::read_family::inline_type_budget::collect(census);
    super::read_result::inline_type_budget::collect(census);
}
