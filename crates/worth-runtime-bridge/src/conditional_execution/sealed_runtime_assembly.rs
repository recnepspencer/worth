mod owned_async;

use worth_signal::facade::branch::SignalOwnerServicePorts;

use super::{
    BridgeAppliedConditionalInstallationExtension, BridgeConditionalDenial,
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
    BridgeOwnedSignalRuntime, BridgePreparedConditionalInstallationExtension,
};
use crate::facade::RuntimeWorldCorrespondencePort;

/// One-time assembly of the Bridge sibling and the exact Signal owner ports
/// issued while that sibling was sealed.
///
/// The contained runtime has no public extraction path. This keeps ordinary
/// Signal services, conditional execution, and Runtime World correspondence
/// tied to one physical owner state.
pub struct BridgeSealedRuntimeAssembly {
    runtime: BridgeOwnedSignalRuntime,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    signal_services: SignalOwnerServicePorts<(), (), (), (), ()>,
    signal_definition_publication: Option<
        worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
            (),
            (),
            (),
            (),
            (),
        >,
    >,
    signal_basis: worth_signal::facade::branch::AdmittedSignalBranchBasis,
    correspondence: RuntimeWorldCorrespondencePort,
    correspondence_basis: crate::facade::AdmittedRuntimeWorldCorrespondenceBasis,
    affinity: BridgeSealedOwnerAffinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BridgeSealedOwnerAffinity {
    signal_graph_instance_id: u64,
}

impl BridgeSealedRuntimeAssembly {
    #[cfg(test)]
    pub(super) fn test_runtime(&self) -> &super::BridgeOwnedSignalRuntime {
        &self.runtime
    }
    pub fn active_semantic_dependency_count(&self) -> usize {
        self.runtime.active_semantic_dependency_count()
    }

    pub fn installed_conditional_definition_count(&self) -> usize {
        self.runtime
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Confirms that a sibling owner was derived from the exact authoritative
    /// source sealed into this assembly.
    pub fn readmits_authoritative_source_profile(
        &self,
        candidate: &crate::facade::BridgeAuthoritativeSourceProfile,
    ) -> bool {
        self.runtime
            .bridge
            .authoritative_source_profile()
            .is_some_and(|installed| installed == candidate)
    }

    pub fn install_managed_clock(
        &self,
        parts: super::BridgeManagedClockInstallationParts<'_>,
    ) -> Result<super::BridgeManagedClockBinding, super::BridgeManagedTemporalDenial> {
        self.runtime.install_managed_clock(parts)
    }

    pub fn close_managed_clock(
        &self,
        binding: super::BridgeManagedClockBinding,
    ) -> Result<super::BridgeManagedClockClosure, super::BridgeManagedTemporalDenial> {
        self.runtime.close_managed_clock(binding)
    }

    pub fn admit_conditional_evaluation(
        &self,
        request: super::BridgeConditionalEvaluationAdmissionRequest<'_>,
    ) -> Result<super::BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.runtime.admit_conditional_evaluation(request)
    }

    pub fn admit_conditional_signal_basis(
        &self,
        lowering: &std::sync::Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<super::BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.runtime.admit_conditional_signal_basis(lowering, basis)
    }

    pub fn admit_exact_conditional_signal_basis(
        &self,
        anchor: &std::sync::Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<super::BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.runtime
            .admit_exact_conditional_signal_basis(anchor, basis)
    }

    pub fn readmit_conditional_evaluation(
        &self,
        request: super::BridgeConditionalEvaluationReadmissionRequest<'_>,
    ) -> Result<super::BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.runtime.readmit_conditional_evaluation(request)
    }

    pub fn execute_admitted_conditional(
        &self,
        session: &super::BridgeConditionalEvaluationSession,
        request: super::BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime
            .execute_admitted_conditional(session, request, compute_context)
    }

    pub(super) fn from_sealed_runtime(
        mut runtime: BridgeOwnedSignalRuntime,
    ) -> Result<Self, BridgeConditionalDenial> {
        let conditional_evaluation_budget = runtime
            .lock_signal_runtime()
            .runtime_policy()
            .conditional_evaluation_budget;
        let signal_services = runtime.signal_services()?.owner_services();
        let signal_definition_publication = runtime
            .signal_services_mut()?
            .take_definition_publication()?;
        let signal_basis = runtime.signal_services()?.issuance_basis();
        let correspondence = runtime.bridge.runtime_world_correspondence_port();
        let correspondence_basis =
            correspondence.admit_runtime_baseline(runtime.owned_signal_graph_instance_id());
        let affinity = BridgeSealedOwnerAffinity {
            signal_graph_instance_id: runtime.owned_signal_graph_instance_id(),
        };
        Ok(Self {
            runtime,
            conditional_evaluation_budget,
            signal_services,
            signal_definition_publication: Some(signal_definition_publication),
            signal_basis,
            correspondence,
            correspondence_basis,
            affinity,
        })
    }

    pub fn signal_owner_services(&self) -> SignalOwnerServicePorts<(), (), (), (), ()> {
        self.signal_services.clone()
    }

    /// The Signal-owner limits installed before this assembly was sealed.
    /// Query uses the slot ceiling to bound idle derived-evaluation custody;
    /// the retained sessions continue to own the actual Signal charges.
    pub const fn conditional_evaluation_budget(
        &self,
    ) -> worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
        self.conditional_evaluation_budget
    }

    /// Move the sole Signal definition-publication capability into Runtime World.
    pub fn take_runtime_world_signal_definition_publication(
        &mut self,
    ) -> Result<
        worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
            (),
            (),
            (),
            (),
            (),
        >,
        BridgeConditionalDenial,
    > {
        self.signal_definition_publication.take().ok_or_else(|| {
            BridgeConditionalDenial::new(
                super::BridgeConditionalDenialKind::SignalExecution,
                "Runtime World already owns the Signal definition-publication capability",
            )
        })
    }

    pub fn runtime_world_correspondence_port(&self) -> RuntimeWorldCorrespondencePort {
        self.correspondence.clone()
    }

    pub fn admitted_runtime_world_correspondence_basis(
        &self,
    ) -> &crate::facade::AdmittedRuntimeWorldCorrespondenceBasis {
        &self.correspondence_basis
    }

    pub fn admitted_signal_basis(
        &self,
    ) -> &worth_signal::facade::branch::AdmittedSignalBranchBasis {
        &self.signal_basis
    }

    pub fn owned_signal_graph_instance_id(&self) -> u64 {
        self.affinity.signal_graph_instance_id
    }

    pub fn prepare_conditional_reconstitution(
        &self,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<super::BridgePreparedConditionalReconstitution, BridgeConditionalDenial> {
        self.runtime.prepare_conditional_reconstitution(basis)
    }

    pub fn activate_conditional_reconstitution(
        &mut self,
        candidate: super::BridgePreparedConditionalReconstitution,
    ) -> Result<super::BridgeConditionalRuntimeReconstitutionReport, BridgeConditionalDenial> {
        self.runtime.activate_conditional_reconstitution(candidate)
    }

    pub fn close_conditional_resources(&mut self) {
        self.runtime.close_conditional_resources();
    }

    #[cfg(test)]
    pub(crate) fn destroy_reconstitutable_indexes_for_test(&mut self) {
        self.runtime.destroy_reconstitutable_indexes_for_test();
    }

    #[doc(hidden)]
    pub fn prepare_owned_conditional_definition_successor(
        &self,
        predecessor: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<BridgePreparedConditionalInstallationExtension, BridgeConditionalDenial> {
        self.runtime
            .prepare_owned_conditional_definition_successor(predecessor, request)
    }

    #[doc(hidden)]
    pub fn apply_owned_conditional_installation_extension<E, Ctx>(
        &self,
        transaction: &mut worth_signal::facade::SignalTransaction<'_, (), (), E, Ctx, ()>,
        prepared: BridgePreparedConditionalInstallationExtension,
    ) -> Result<BridgeAppliedConditionalInstallationExtension, BridgeConditionalDenial> {
        self.runtime
            .apply_owned_conditional_installation_extension(transaction, prepared)
    }

    #[doc(hidden)]
    pub fn complete_owned_conditional_installation_activation(
        &self,
        applied: &mut BridgeAppliedConditionalInstallationExtension,
        binding: worth_signal::facade::branch::SignalConditionalDefinitionAdvanceBinding,
    ) -> Result<(), BridgeConditionalDenial> {
        self.runtime
            .complete_owned_conditional_installation_activation(applied, binding)
    }

    #[doc(hidden)]
    pub fn commit_owned_conditional_installation_extension(
        &self,
        applied: BridgeAppliedConditionalInstallationExtension,
    ) -> std::sync::Arc<BridgeInstalledConditionalLowering> {
        self.runtime
            .commit_owned_conditional_installation_extension(applied)
    }

    pub fn reconcile_managed_temporal_intent(
        &self,
        parts: super::BridgeManagedTemporalIntentReconciliationParts<'_>,
    ) -> Result<super::BridgeManagedTemporalIntentReconciliation, super::BridgeManagedTemporalDenial>
    {
        self.runtime.reconcile_managed_temporal_intent(parts)
    }

    pub fn observe_managed_clock(
        &self,
        parts: super::BridgeManagedClockObservationParts<'_>,
    ) -> Result<super::BridgeManagedClockObservationOutcome, super::BridgeManagedTemporalDenial>
    {
        self.runtime.observe_managed_clock(parts)
    }

    pub fn execute_managed_due_wake(
        &self,
        request: super::BridgeManagedConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime
            .execute_managed_due_wake(request, compute_context)
    }

    pub fn deliver_authoritative_change(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        self.runtime
            .deliver_authoritative_change(signal_basis, dependency_ordinal, request)
    }

    pub fn conditional_lifecycle_probe(&self) -> super::BridgeConditionalRuntimeLifecycleProbe {
        self.runtime.conditional_lifecycle_probe()
    }

    pub fn execute(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: super::BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime.execute(signal_basis, request, compute_context)
    }

    pub fn reenter_retained_conditional_decision(
        &self,
        request: super::BridgeConditionalDecisionReentryRequest<'_>,
    ) -> Result<super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime.reenter_retained_conditional_decision(request)
    }

    pub fn revoke_conditional_liveness(&mut self) {
        self.runtime.revoke_conditional_liveness();
    }

    pub fn deliver_owned_authoritative_change(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        self.runtime
            .deliver_owned_authoritative_change(signal_basis, dependency_ordinal)
    }

    pub fn retire_owned_conditional(
        &mut self,
        lowering: &std::sync::Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<(), BridgeConditionalDenial> {
        self.runtime.retire_owned_conditional(lowering)
    }

    pub fn owned_signal_active_node_count(&self) -> Result<usize, BridgeConditionalDenial> {
        self.runtime.owned_signal_active_node_count()
    }
}
