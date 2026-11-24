//! Exact selected graph, canonical meaning and its retained funding.
use crate::branch::model_ownership::RelationalOwnedModelGraphSelection;
use crate::history::data::CanonicalOwnedGraphSelection;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct PreparedOwnedGraphSelection {
    pub(crate) graph: Arc<RelationalOwnedModelGraphSelection>,
    pub(crate) meaning: Arc<CanonicalOwnedGraphSelection>,
    pub(crate) _funding: Arc<Option<worth_execution::ExecutionMemoryReservation>>,
}
