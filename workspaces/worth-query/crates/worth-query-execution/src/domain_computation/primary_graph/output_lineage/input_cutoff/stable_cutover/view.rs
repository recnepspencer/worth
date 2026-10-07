//! Read-only views of one owner-published stable lineage row.

use super::{PublishedStableLineage, RecordedOutput};
use std::sync::Arc;
use worth_runtime_world::facade::ProductBranchObservation;

impl PublishedStableLineage {
    /// The filled row a reader at `observation` selected.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn selected_at(
        cell: &Arc<std::sync::OnceLock<RecordedOutput>>,
        observation: &ProductBranchObservation,
    ) -> Self {
        Self {
            cell: Arc::clone(cell),
            observation: observation.clone(),
        }
    }
    pub(super) fn recorded(&self) -> &RecordedOutput {
        self.cell
            .get()
            .expect("stable publication filled its immutable row")
    }
    pub(in crate::domain_computation::primary_graph) fn same_publication(
        &self,
        other: &Self,
    ) -> bool {
        Arc::ptr_eq(&self.cell, &other.cell) && self.observation == other.observation
    }
    pub(in crate::domain_computation::primary_graph) fn exact_settlement(
        &self,
    ) -> &Arc<super::super::super::RecordedSettlementIdentity> {
        &self.recorded().settlement_identity
    }
    pub(in crate::domain_computation::primary_graph) fn output_correspondence(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>
    {
        &self.recorded().correspondence
    }
    pub(in crate::domain_computation::primary_graph) fn source_identity(
        &self,
    ) -> Option<super::super::super::RecordedSourceIdentity> {
        self.recorded().source_identity
    }
    pub(in crate::domain_computation::primary_graph) fn source_scope(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        self.recorded().settlement_identity.source().scope
    }
    pub(in crate::domain_computation::primary_graph) fn source_partition_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.recorded().source_partition_identity
    }
    pub(in crate::domain_computation::primary_graph) fn producer_dependency_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.recorded().producer_dependency_identity
    }
    pub(in crate::domain_computation::primary_graph) fn idempotency_key_identity(
        &self,
    ) -> [u8; 32] {
        self.recorded().idempotency_key_identity
    }
    pub(in crate::domain_computation::primary_graph) fn checkpoint_source_facts(
        &self,
    ) -> Result<
        Option<Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>>,
        worth_relational::facade::durability::DurabilityError,
    > {
        self.recorded().checkpoint_source_facts()
    }
    pub(in crate::domain_computation::primary_graph) fn observation(
        &self,
    ) -> &ProductBranchObservation {
        &self.observation
    }
}
