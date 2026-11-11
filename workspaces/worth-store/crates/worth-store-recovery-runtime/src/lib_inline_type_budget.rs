//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::entry::inline_type_budget::collect(census);
    super::handoff::inline_type_budget::collect(census);
    super::orchestration::inline_type_budget::collect(census);
}
