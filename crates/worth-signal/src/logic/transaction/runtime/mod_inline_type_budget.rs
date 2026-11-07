//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::config::inline_type_budget::collect(census);
    super::state::inline_type_budget::collect(census);
    super::transaction::inline_type_budget::collect(census);
}
