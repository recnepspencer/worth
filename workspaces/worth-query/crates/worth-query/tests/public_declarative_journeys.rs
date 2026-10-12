//! Downstream journeys through the ordinary declarative and installed-domain facades.
#[path = "support/execution_policy.rs"]
mod test_execution_policy;

// The two downstream targets intentionally select different fixtures from the
// same non-product support tree until Phase 6/7 gives them package ownership.
#[allow(dead_code, unused_imports)]
mod support;

mod causal_inspection_public_dx;
mod declarative_history_comparison_public_dx;
mod declarative_inspection_public_dx;
mod declarative_live_public_dx;
mod declarative_product_boundary_certification;
mod declarative_read_public_dx;
mod declarative_workflow_public_dx;
mod installed_domain_facade_extension;
mod native_aspect_mutation_public_dx;
