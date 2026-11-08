//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::initial_empty_stop::inline_type_budget::collect(census);
    super::branch_cell_state::inline_type_budget::collect(census);
    super::branch_registry::inline_type_budget::collect(census);
    super::cancellation::inline_type_budget::collect(census);
    super::conditional_execution::inline_type_budget::collect(census);
    super::counters::inline_type_budget::collect(census);
    super::lifecycle_state::inline_type_budget::collect(census);
    super::operation_control::inline_type_budget::collect(census);
    super::owner::inline_type_budget::collect(census);
    super::owner_metadata::inline_type_budget::collect(census);
}
