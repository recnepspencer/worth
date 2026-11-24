//! Exact selected graph, canonical meaning and its retained funding.
use crate::branch::model_ownership::RelationalOwnedModelGraphSelection;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PreparedOwnedGraphSelection {
    pub graph: Arc<RelationalOwnedModelGraphSelection>,
    pub meaning: Arc<CanonicalOwnedGraphSelection>,
    pub _funding: Arc<Option<worth_execution::ExecutionMemoryReservation>>,
}
