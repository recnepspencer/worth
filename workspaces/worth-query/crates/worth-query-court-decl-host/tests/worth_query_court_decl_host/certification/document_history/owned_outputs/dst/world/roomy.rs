//! A world with room for deep histories: the correction cost court records
//! 72 steps on one document, past the default world's branch capacity.
use super::*;
use std::num::NonZeroUsize;
use worth_query_testkit_document_retention::facade::{
    document_retention_model::host::{primary_graph, runtime, WorthQueryApplicationLimits},
    document_world_resources::{world_resources_with_capacity, WorldLifecycleCapacity},
};

/// [`install`] with the room `history_envelope::output_forest::step_work`
/// measures deep histories in.
pub fn install_roomy() -> Host {
    install_with(
        Shape::Members,
        WorthQueryApplicationLimits::new(
            world_resources_with_capacity(
                1024,
                128 * 1024 * 1024,
                1024,
                WorldLifecycleCapacity {
                    live_product_branches: 128,
                    active_observations: 1024,
                    owner_created_component_custody_records: 256,
                },
                WorthQueryAdmissionClock::system(),
            ),
            runtime::WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(128).unwrap()),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
    )
}
