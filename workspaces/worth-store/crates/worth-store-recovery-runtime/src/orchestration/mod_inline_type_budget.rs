//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::handoff::inline_type_budget::collect(census);
    super::publication::inline_type_budget::collect(census);
    super::staging::inline_type_budget::collect(census);
}
