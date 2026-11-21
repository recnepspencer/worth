//! Exact relation lifecycle material retained by the original commit owner.
use std::sync::Arc;
use worth_query_installation::facade::InstalledPreImageRelationLocus;
use worth_relational::facade::{mvcc::ValidatedRelationMutation, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRetainedPreImageRelation {
    locus: InstalledPreImageRelationLocus,
    mutation: ValidatedRelationMutation,
}
impl WorthQueryRetainedPreImageRelation {
    pub fn locus(&self) -> &InstalledPreImageRelationLocus {
        &self.locus
    }
    pub fn mutation(&self) -> &ValidatedRelationMutation {
        &self.mutation
    }
}
#[derive(Debug)]
pub struct RetainedRelations {
    rows: Vec<WorthQueryRetainedPreImageRelation>,
    loci: Vec<InstalledPreImageRelationLocus>,
    _footprint: worth_relational::facade::mvcc::ValidatedRelationMutationFootprint,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PartialEq for RetainedRelations {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.loci == other.loci
    }
}
impl Eq for RetainedRelations {}
impl RetainedRelations {
    pub fn from_parts(
        rows: Vec<(worth_query_installation::facade::InstalledPreImageRelationLocus, ValidatedRelationMutation)>,
        loci: Vec<worth_query_installation::facade::InstalledPreImageRelationLocus>,
        footprint: worth_relational::facade::mvcc::ValidatedRelationMutationFootprint,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Arc::new(Self {
            rows: rows
                .into_iter()
                .map(|(locus, mutation)| WorthQueryRetainedPreImageRelation { locus, mutation })
                .collect(),
            loci,
            _footprint: footprint,
            _memory: memory,
        })
    }
    pub fn loci(&self) -> &[InstalledPreImageRelationLocus] {
        &self.loci
    }
    pub fn rows(&self) -> &[WorthQueryRetainedPreImageRelation] {
        &self.rows
    }
}
pub use worth_query_execution_contracts::facade::{
    WorthQueryRelationPreImageDenial, WorthQueryRelationPreImageFailure,
};
