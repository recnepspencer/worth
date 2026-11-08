//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::contract::inline_type_budget::collect(census);
    super::decision_evidence::inline_type_budget::collect(census);
    super::denial::inline_type_budget::collect(census);
    super::evaluation_session::inline_type_budget::collect(census);
    super::execution::inline_type_budget::collect(census);
    super::managed_wake_execution::inline_type_budget::collect(census);
    super::observation_retention::inline_type_budget::collect(census);
    super::output_comparison::inline_type_budget::collect(census);
    super::owned_async::inline_type_budget::collect(census);
    super::providers::inline_type_budget::collect(census);
    super::retention::inline_type_budget::collect(census);
    super::runtime_builder::inline_type_budget::collect(census);
    super::sealed_runtime_assembly::inline_type_budget::collect(census);
    super::semantic_contract::inline_type_budget::collect(census);
    super::signal_basis_binding::inline_type_budget::collect(census);
    super::signal_denial::inline_type_budget::collect(census);
}
