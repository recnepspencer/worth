//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::WorthQueryApplicationManagedCleanup::observe_inline_budget(census);
    super::commit_outcome::inline_type_budget::collect(census);
    super::commit_receipt::inline_type_budget::collect(census);
}
