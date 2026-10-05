use std::any::TypeId;

use worth_runtime_world::facade::PlannedProductReferenceSuccessor;

use super::{denial, PreparedOutputLineageSlot};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Input proof data moves across a World recovery plan. The cancelled address
/// and its retained capacity stay with the original slot's Drop path.
pub(in crate::domain_computation::primary_graph) struct PreparedLineageRecoveryMetadata {
    handler: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    >,
    decision_reuse: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedDecisionReuseProof,
    >,
    reuse: Option<crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey>,
}

impl PreparedOutputLineageSlot {
    /// Reuse is allowed only for this attempt's exact still-vacant address.
    pub(in crate::domain_computation::primary_graph) fn matches_planned_successor(
        &self,
        scope: &WorthQueryOperationScopeBinding,
        output_binding: TypeId,
        partition: Option<[u8; 32]>,
        planned: &PlannedProductReferenceSuccessor,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(12)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?;
        Ok(!self.filled
            && self.record_cell.get().is_none()
            && self.partition_cell.get().is_none()
            && self.source.runtime_authority == scope.runtime_authority()
            && &self.source.schema == scope.binding_identity()
            && self.source.scope == scope.scope()
            && self.source.output_binding == output_binding
            && self.coordinate.occurrence == planned.occurrence()
            && self.coordinate.generation == planned.generation().get()
            && self.partition == partition
            && self.identity.source() == &self.source
            && self.identity.address()
                == (
                    self.coordinate.occurrence,
                    self.coordinate.generation,
                    self.identity.slot(),
                ))
    }

    pub(in crate::domain_computation::primary_graph) fn into_recovery_metadata(
        mut self,
    ) -> PreparedLineageRecoveryMetadata {
        let metadata = PreparedLineageRecoveryMetadata {
            handler: self.completed_handler_facts.take(),
            decision_reuse: self.completed_decision_reuse.take(),
            reuse: self.prepared_input_reuse_key.take(),
        };
        drop(self);
        metadata
    }

    pub(in crate::domain_computation::primary_graph) fn retain_recovery_metadata(
        &mut self,
        mut metadata: PreparedLineageRecoveryMetadata,
    ) {
        assert!(self.completed_handler_facts.is_none());
        assert!(self.completed_decision_reuse.is_none());
        assert!(self.prepared_input_reuse_key.is_none());
        self.completed_handler_facts = metadata.handler.take();
        self.completed_decision_reuse = metadata.decision_reuse.take();
        self.prepared_input_reuse_key = metadata.reuse.take();
    }
}
