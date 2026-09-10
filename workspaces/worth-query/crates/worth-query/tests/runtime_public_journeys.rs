//! Downstream journeys through runtime bootstrap, graph composition, and submission facades.

// The two downstream targets intentionally select different fixtures from the
// same non-product support tree until Phase 6/7 gives them package ownership.
#[allow(dead_code, unused_imports)]
mod support;

mod declaration_authority_runtime;
mod graph_composition_public_bridge;
mod graph_composition_public_bridge_existing;
mod in_memory_test_backend_facade;
mod product_branch_lifecycle;
mod public_bridge_runtime_bootstrap;
mod public_submission_lane_replacements;
mod runtime_backed_read_bootstrap;
