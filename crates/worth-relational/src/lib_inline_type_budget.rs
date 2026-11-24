//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::authority::commit::inline_type_budget::collect(census);
    super::mvcc::inline_type_budget::collect(census);
    super::runtime::inline_type_budget::collect(census);
}
