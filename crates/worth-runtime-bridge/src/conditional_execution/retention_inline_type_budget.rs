//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::ledger::inline_type_budget::collect(census);
    super::trigger::inline_type_budget::collect(census);
}
