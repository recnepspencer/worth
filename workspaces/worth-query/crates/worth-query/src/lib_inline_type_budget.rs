//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::domain_installation::inline_type_budget::collect(census);
    super::intent_admission::inline_type_budget::collect(census);
    super::live::inline_type_budget::collect(census);
    super::memory_workspace::inline_type_budget::collect(census);
    super::runtime::inline_type_budget::collect(census);
}
