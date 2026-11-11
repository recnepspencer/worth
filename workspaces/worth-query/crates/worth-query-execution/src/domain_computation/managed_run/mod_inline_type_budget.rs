//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::direct_admission_failure::inline_type_budget::collect(census);
    super::terminal::inline_type_budget::collect(census);
    super::truth_read_request::inline_type_budget::collect(census);
}
