//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::collection_window::inline_type_budget::collect(census);
    super::conditional_execution::inline_type_budget::collect(census);
    super::consumer_support::inline_type_budget::collect(census);
    super::graph_participation::inline_type_budget::collect(census);
    super::installation_state::inline_type_budget::collect(census);
    super::operation_execution::inline_type_budget::collect(census);
}
