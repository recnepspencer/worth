//! Read-only views of one owner-published stable lineage row.

use super::{PublishedStableLineage, RecordedOutput};
use std::sync::Arc;
use worth_runtime_world::facade::ProductBranchObservation;

impl PublishedStableLineage {
    pub fn has_executed_adoption_support(&self) -> bool {
        self.recorded().computation_source.is_executed_adoption()
    }

    /// The filled row a reader at `observation` selected.
    pub fn selected_at(
        cell: &Arc<std::sync::OnceLock<RecordedOutput>>,
        observation: &ProductBranchObservation,
    ) -> Self {
        Self {
            cell: Arc::clone(cell),
            observation: observation.clone(),
        }
    }
    pub fn recorded(&self) -> &RecordedOutput {
        self.cell
            .get()
            .expect("stable publication filled its immutable row")
    }
    pub fn same_publication(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cell, &other.cell) && self.observation == other.observation
    }
    pub fn exact_settlement(&self) -> &Arc<super::super::super::RecordedSettlementIdentity> {
        &self.recorded().settlement_identity
    }
    pub fn output_correspondence(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>
    {
        &self.recorded().correspondence
    }
    pub fn source_identity(&self) -> Option<super::super::super::RecordedSourceIdentity> {
        self.recorded().source_identity
    }
    pub fn source_scope(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        self.recorded().settlement_identity.source().scope
    }
    pub fn source_partition_identity(&self) -> Option<[u8; 32]> {
        self.recorded().source_partition_identity
    }
    pub fn producing_implementation(
        &self,
    ) -> Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation> {
        self.recorded().producing_implementation
    }

    pub fn producer_dependency_identity(&self) -> Option<[u8; 32]> {
        self.recorded().producer_dependency_identity
    }
    pub fn idempotency_key_identity(&self) -> [u8; 32] {
        self.recorded().idempotency_key_identity
    }
    pub fn checkpoint_source_facts(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>
    {
        self.recorded().checkpoint_source_facts()
    }
    /// Borrow the already sealed output expectations from the selected row.
    /// Capture must carry these alongside producer facts, never reread a new
    /// head to reconstruct what the original output expected.
    pub fn checkpoint_output_witness(
        &self,
    ) -> Option<&super::super::super::SealedNativeOutputWitness> {
        self.recorded().native_output_witness()
    }
    pub fn observation(&self) -> &ProductBranchObservation {
        &self.observation
    }
}

mod court_doors_for_test;
