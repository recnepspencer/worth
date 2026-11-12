//! Borrowed historical metadata grants no Native root, current dependency or authority.
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, WorthQueryApplicationFactKey,
};
use std::sync::Arc;
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor},
    history::CommitId,
    mvcc::RelationalRequiredOutputPublication,
};
/// Original read coordinates and keys, available only through admitted source inspection.
/// These process-local diagnostics have no wire representation or selection conversion.
pub struct WorthQueryHistoricalProducerRead {
    publication: (u64, usize),
    basis: RelationalBranchBasisDescriptor,
    _applied:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    operation: &'static str,
    keys: Arc<ComputationFactAttribution>,
}
impl WorthQueryHistoricalProducerRead {
    pub(in crate::domain_computation::primary_graph) fn from_read(
        publication: &RelationalRequiredOutputPublication,
        basis: &AdmittedRelationalBranchBasis,
        operation: &'static str,
        keys: Arc<ComputationFactAttribution>,
    ) -> Self {
        Self {
            publication: publication.authority().publication_coordinate(),
            basis: basis.descriptor().clone(),
            _applied: None,
            operation,
            keys,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn from_input(
        publication: &RelationalRequiredOutputPublication,
        input: &worth_relational::facade::mvcc::RelationalPredecessorInput,
        operation: &'static str,
        keys: Arc<ComputationFactAttribution>,
    ) -> Self {
        let mut evidence = Self::from_read(publication, input.before_basis(), operation, keys);
        evidence._applied = input
            .applied_read()
            .map(|read| read.candidate_identity().qualification());
        evidence
    }
    pub fn publication_commit(&self) -> CommitId {
        CommitId(self.publication.0)
    }
    pub fn publication_slot(&self) -> usize {
        self.publication.1
    }
    pub fn basis(&self) -> &RelationalBranchBasisDescriptor {
        &self.basis
    }
    pub fn operation(&self) -> &str {
        self.operation
    }
    /// Original historical keys, never the producer's current C dependencies.
    pub fn keys(&self) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.keys.historical_handler_keys()
    }
}
