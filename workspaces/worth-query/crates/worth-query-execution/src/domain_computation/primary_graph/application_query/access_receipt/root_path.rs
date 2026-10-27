//! Descriptive executed-step evidence; no graph identities or read authority.
use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;
use worth_execution::ExecutionMemoryReservation;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, RelationDirection,
};

/// Actual acquired reads for one executed authored root-path step.
/// Indices are zero-based within the query's paths and steps. This descriptive
/// evidence carries no graph identities, read authority or traversal capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryRootPathStepRead {
    path_index: usize,
    step_index: usize,
    traversal: ApplicationQueryRootPathTraversal,
    direction: RelationDirection,
    reads: WorthQueryInvariantProjectionWork,
}
impl WorthQueryApplicationQueryRootPathStepRead {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        path_index: usize,
        step_index: usize,
        traversal: ApplicationQueryRootPathTraversal,
        direction: RelationDirection,
        reads: WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            path_index,
            step_index,
            traversal,
            direction,
            reads,
        }
    }
    /// Zero-based authored path index.
    pub const fn path_index(&self) -> usize {
        self.path_index
    }
    /// Zero-based authored step index; guard positions retain their usual meaning.
    pub const fn step_index(&self) -> usize {
        self.step_index
    }
    pub const fn traversal(&self) -> ApplicationQueryRootPathTraversal {
        self.traversal
    }
    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }
    pub const fn reads(&self) -> WorthQueryInvariantProjectionWork {
        self.reads
    }
}

pub(in crate::domain_computation::primary_graph::application_query) struct RootPathReadEvidence {
    values: Vec<WorthQueryApplicationQueryRootPathStepRead>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RootPathReadEvidence {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        values: Vec<WorthQueryApplicationQueryRootPathStepRead>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            values,
            _memory: memory,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn values(
        &self,
    ) -> &[WorthQueryApplicationQueryRootPathStepRead] {
        &self.values
    }
}
impl PartialEq for RootPathReadEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for RootPathReadEvidence {}
impl std::fmt::Debug for RootPathReadEvidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.values.fmt(f)
    }
}
