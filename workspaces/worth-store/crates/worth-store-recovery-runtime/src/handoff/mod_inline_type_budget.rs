//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::recovered::inline_type_budget::collect(census);
    super::recovered_evidence::inline_type_budget::collect(census);
}
