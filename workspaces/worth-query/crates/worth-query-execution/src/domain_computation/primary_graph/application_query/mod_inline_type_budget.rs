//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::history::inline_type_budget::collect(census);
    super::live::inline_type_budget::collect(census);
    super::retained_read::inline_type_budget::collect(census);
}
