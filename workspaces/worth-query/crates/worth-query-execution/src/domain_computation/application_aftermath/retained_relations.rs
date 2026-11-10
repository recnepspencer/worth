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
pub(super) struct RetainedRelations {
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
    pub(super) fn from_seal(
        seal: crate::domain_computation::primary_graph::WorthQueryRetainedRelationPreImageSeal,
    ) -> Arc<Self> {
        let (rows, loci, footprint, memory) = seal.into_parts();
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
    pub(super) fn loci(&self) -> &[InstalledPreImageRelationLocus] {
        &self.loci
    }
    pub(super) fn rows(&self) -> &[WorthQueryRetainedPreImageRelation] {
        &self.rows
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRelationPreImageDenial {
    MissingObservation,
    MissingMutation,
    ExceedsByteBound,
    UnsupportedVisibility,
    IncompleteInverse,
    ForeignEndpoint,
    ExtraInverse,
    RepeatedInverse,
    RestoredFieldMismatch,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRelationPreImageFailure {
    kind: WorthQueryRelationPreImageDenial,
    affected_records: Vec<RecordRef>,
}
impl WorthQueryRelationPreImageFailure {
    pub fn kind(&self) -> WorthQueryRelationPreImageDenial {
        self.kind
    }
    pub fn affected_records(&self) -> &[RecordRef] {
        &self.affected_records
    }
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryRelationPreImageDenial,
        affected_records: Vec<RecordRef>,
    ) -> Self {
        Self {
            kind,
            affected_records,
        }
    }
}
