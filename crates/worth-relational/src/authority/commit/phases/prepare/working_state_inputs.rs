//! Exact inputs for cloning the authoritative working state.
use crate::config::data::AdjacencyPolicy;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub(crate) struct WorkingStatePreparation<'a> {
    pub(crate) adjacency_policy: &'a AdjacencyPolicy,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}
