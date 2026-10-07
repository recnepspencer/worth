//! Routes a failed stage completion without exposing its live run authority.

use crate::basis_lifecycle::BasisOperationLane;
use worth_proof::TransitionOutcome;

use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind,
    WorthQueryWorkflowAdvanceOutcome, WorthQueryWorkflowRun,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(in crate::domain_installation::operation_execution) fn outcome_from_denial(
        mut self,
        mut denial: WorthQueryWorkflowAdvanceDenial,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        use WorthQueryWorkflowAdvanceDenialKind as Kind;
        let (stale, rebind, failed) = match denial.kind() {
            Kind::RuntimeAuthority(cause) => (
                *cause == crate::domain_installation::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
                *cause == crate::domain_installation::WorthQueryDomainHandleDenialKind::PackageIdentityChanged,
                false,
            ),
            Kind::ArtifactCarriage(artifact) => (
                artifact.kind() == crate::domain_installation::WorthQueryArtifactDenialKind::StaleInstallationGeneration,
                false,
                false,
            ),
            Kind::StageExecutor { .. }
            | Kind::UndeclaredFailureClass(_)
            | Kind::PredecessorAuthorityMissing(_)
            | Kind::ResourceAdmissionMissing
            | Kind::ConditionalExecution(_)
            | Kind::ComputationPanic { .. }
            | Kind::ComputationResultCapacity { .. }
            | Kind::ComputationNestedStopped { .. } => (false, false, true),
            Kind::UnknownStage
            | Kind::StageAlreadyCompleted
            | Kind::PredecessorIncomplete(_)
            | Kind::RequiredCapability(_)
            | Kind::RequiredDomain(_)
            | Kind::InputContract
            | Kind::GraphProvider(_)
            | Kind::PrimaryReadEvidence
            | Kind::EffectEvidence
            | Kind::InvariantEvidence
            | Kind::LineageEvidence
            | Kind::CostContract
            | Kind::OutputContract
            | Kind::TerminalContract
            | Kind::DomainEvidence(_)
            | Kind::ComputationAdmission(_)
            | Kind::ComputationWorkExhausted { .. }
            | Kind::ComputationCancelled { .. }
            | Kind::ComputationDeadline { .. }
            | Kind::ParallelFrontierShape
            | Kind::NonDeterministicLowering
            | Kind::ParallelProvider(_)
            | Kind::ParallelNotAdmitted(_)
            | Kind::ConditionalReentry(_) => (false, false, false),
        };
        for receipt in self.receipts.iter_mut().rev() {
            receipt.cancel_artifact_output();
        }
        self.artifact_registry.close_cancelled();
        if !denial.has_managed_cleanup() {
            if let Some(running) = self.managed.take() {
                denial = denial.with_managed_cleanup(running.abandon().cleanup());
            }
        }
        let completed_effects = self
            .receipts
            .iter()
            .flat_map(|receipt| receipt.effect_evidence().iter().cloned())
            .collect();
        let stop = denial
            .prepend_executed_effects(completed_effects)
            .with_completed_stage_receipts(self.receipts);
        if stale {
            TransitionOutcome::Stale(stop)
        } else if rebind {
            TransitionOutcome::RebindRequired(stop)
        } else if failed {
            TransitionOutcome::Failed(stop)
        } else {
            TransitionOutcome::Denied(stop)
        }
    }
}
