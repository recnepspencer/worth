#![deny(private_interfaces)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan;
use crate::domain_computation::provider_session::readmission::{
    WorthQueryWorkflowResourceReadmissionPostProvider,
    WorthQueryWorkflowResourceReadmissionPreProvider,
};
use crate::domain_computation::{
    WorthQueryExecutionResourceAttemptEvidence, WorthQueryWorkflowExecutionResourceAttempt,
};

use super::super::provider_work::{
    WorthQueryManagedProviderCleanupAuthority, WorthQueryManagedProviderWorkEvidence,
    WorthQueryManagedProviderWorkLedger,
};
use super::super::readmission::{
    WorthQueryWorkflowReadmissionProgressionPermit, WorthQueryWorkflowReadmissionRecoveryPermit,
};

static NEXT_WORKFLOW_LOGICAL_RUN: AtomicU64 = AtomicU64::new(1);

mod cleanup;
mod readmission;
mod terminal;

pub use cleanup::{
    WorthQueryWorkflowAffinityCleanupReceipt, WorthQueryWorkflowYieldReleasePending,
};
pub use terminal::WorthQueryWorkflowRunTerminalAffinity;

pub struct WorthQueryWorkflowRunTransitionPermit {
    _owner: (),
}

impl WorthQueryWorkflowRunTransitionPermit {
    const fn mint() -> Self {
        Self { _owner: () }
    }
}

/// Move-only association between one logical workflow run, its current live
/// resource attempt, and the provider-work ledger bound to that attempt.
pub struct WorthQueryWorkflowRunAffinity {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowExecutionResourceAttempt,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub struct WorthQueryWorkflowRunReadmissionPending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowResourceReadmissionPreProvider,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub struct WorthQueryWorkflowRunRestoredPending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowResourceReadmissionPostProvider,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub enum WorthQueryWorkflowRunProviderRestoreOutcome {
    Pending {
        affinity: WorthQueryWorkflowRunRestoredPending,
        provider: super::super::provider_restore::WorthQueryManagedGraphRestorePending,
    },
    Denied {
        affinity: WorthQueryWorkflowRunRestoredPending,
        denial: super::super::provider_restore::WorthQueryManagedGraphRestoreDenied,
    },
    RecoveryRequired {
        affinity: WorthQueryWorkflowRunRestoredPending,
        recovery: super::super::provider_restore::WorthQueryManagedGraphRestoreRecoveryRequired,
    },
}

impl WorthQueryWorkflowRunAffinity {
    pub fn provider_session_matches_attempt(
        attempt: &WorthQueryWorkflowExecutionResourceAttempt,
    ) -> bool {
        attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
            .attempt_identity()
            == attempt.attempt_identity().as_str()
    }

    pub fn initial(attempt: WorthQueryWorkflowExecutionResourceAttempt) -> Self {
        let ordinal = NEXT_WORKFLOW_LOGICAL_RUN.fetch_add(1, Ordering::Relaxed);
        let provider_work = WorthQueryManagedProviderWorkLedger::new(
            attempt
                .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
                .closed_identity(),
        );
        Self {
            logical: Arc::from(format!("managed-logical-run:{ordinal}")),
            attempt,
            provider_work,
        }
    }

    pub fn release_initial(
        self,
    ) -> crate::domain_computation::WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        self.attempt.release()
    }

    pub fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub fn attempt_description_arc(&self) -> &Arc<str> {
        self.attempt.attempt_identity().description_arc()
    }

    pub fn belongs_to_operation(
        &self,
        operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        self.attempt.binding_authority().binding_identity() == operation.binding_identity()
    }

    pub fn retained_capacity_reservation_count(&self) -> usize {
        self.attempt.retained_capacity_reservation_count()
    }

    pub fn operation_resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan
    {
        self.attempt.operation_resources()
    }

    pub fn resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan
    {
        self.attempt.resources()
    }

    pub fn evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        self.attempt.evidence()
    }

    pub(super) fn provider_work_mut(&mut self) -> &mut WorthQueryManagedProviderWorkLedger {
        &mut self.provider_work
    }

    pub fn record_provider_execution_release(
        &mut self,
        evidence: &crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence,
    ) {
        self.provider_work
            .record_provider_execution_release(evidence);
    }

    pub fn abandon_provider_work(&mut self) {
        self.provider_work.abandon();
    }

    pub fn interrupt_provider_step_call(&mut self) {
        self.provider_work.interrupt_step_call();
    }

    pub fn settle_provider_artifacts(
        &mut self,
        evidence: crate::domain_computation::artifact_owner::WorthQueryArtifactOccurrenceSnapshot,
    ) {
        self.provider_work.settle_artifacts(evidence);
    }

    pub fn provider_work_snapshot(&self) -> WorthQueryManagedProviderWorkEvidence {
        self.provider_work.snapshot()
    }

    pub fn provider_work_has_uncertainty(&self) -> bool {
        self.provider_work.has_uncertainty()
    }

    pub fn provider_session_description(&self) -> &str {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
            .identity()
    }

    pub(super) fn provider_plan_session(
        &self,
        _owner: &super::WorthQueryWorkflowProviderPlanPermit,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
    }

    pub(super) fn provider_plan_operation(
        &self,
        _owner: &super::WorthQueryWorkflowProviderPlanPermit,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub(super) fn installation_is_current(&self) -> bool {
        self.attempt
            .binding_authority()
            .is_current_installation_generation()
    }

    pub fn binding_identity_projection(&self) -> &str {
        self.attempt.binding_authority().binding_identity()
    }

    pub fn operation_identity_projection(&self) -> &str {
        self.attempt.binding_authority().operation_identity()
    }

    pub fn basis_identity_projection(&self) -> &str {
        self.attempt.binding_authority().basis_identity()
    }

    pub fn installation_generation_projection(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstallationGeneration {
        self.attempt.binding_authority().installation_generation()
    }

    pub fn belongs_to_runtime(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt.binding_authority().belongs_to(runtime)
    }

    pub fn belongs_to_current_installation(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt
            .binding_authority()
            .belongs_to_current_installation(runtime)
    }

    pub fn preflight_retained_provider_call(
        &self,
        call: &crate::domain_computation::WorthQueryGraphProviderCall,
        stage_evidence: &WorthQueryExecutionResourceAttemptEvidence,
    ) -> Result<
        WorthQueryGraphProviderCallReadmissionPlan,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        call.preflight_readmission(self.attempt.binding_authority(), stage_evidence)
    }

    pub(super) fn bind_graph_provider_call(
        &self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: crate::domain_computation::WorthQueryGraphProviderCallRequest,
        evidence: &WorthQueryExecutionResourceAttemptEvidence,
        resources: Arc<worth_query_installation::facade::WorthQueryExecutionResourceEnvelope>,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphProviderCall,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
            .bind_graph_provider_call(graph_authority, request, evidence, resources)
    }

    pub fn bind_graph_commit_call(
        &self,
        graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
        request: crate::domain_computation::WorthQueryGraphCommitCallRequest,
        evidence: &WorthQueryExecutionResourceAttemptEvidence,
        resources: Arc<worth_query_installation::facade::WorthQueryExecutionResourceEnvelope>,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphCommitCall,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
            .bind_graph_commit_call(graph_authorities, request, evidence, resources)
    }

    pub fn completed_evidence_authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub fn completed_evidence_session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
    }

    pub fn bind_managed_workflow_artifacts(
        &self,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority,
        crate::domain_computation::WorthQueryArtifactDenial,
    > {
        self.attempt
            .bind_workflow_artifacts_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
    }

    pub fn binding_authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub fn managed_stage_resources_and_evidence(
        &self,
        stage_identity: &str,
    ) -> Option<(
        Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )>{
        self.attempt.stage_resources_and_evidence_for_managed_run(
            stage_identity,
            &WorthQueryWorkflowRunTransitionPermit::mint(),
        )
    }

    pub(super) fn stage_graph_support_matches(
        &self,
        stage_identity: &str,
        role: &str,
        expected: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    ) -> bool {
        self.attempt
            .resources()
            .stage(stage_identity)
            .and_then(|stage| stage.support_snapshot().graph_provider(role))
            == Some(expected)
    }

    pub fn begin_readmission(
        self,
        stage_resources: Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        call: WorthQueryGraphProviderCallReadmissionPlan,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowRunReadmissionPending {
        WorthQueryWorkflowRunReadmissionPending {
            logical: self.logical,
            attempt: WorthQueryWorkflowResourceReadmissionPreProvider::begin(
                self.attempt,
                stage_resources,
                call,
                &WorthQueryWorkflowRunTransitionPermit::mint(),
            ),
            provider_work: self.provider_work,
        }
    }

    pub fn into_terminal_parts(
        self,
    ) -> (
        WorthQueryWorkflowRunTerminalAffinity,
        WorthQueryManagedProviderWorkEvidence,
        WorthQueryManagedProviderCleanupAuthority,
    ) {
        let (provider_work, cleanup) = self.provider_work.into_terminal_parts();
        (
            WorthQueryWorkflowRunTerminalAffinity::new(self.logical, self.attempt),
            provider_work,
            cleanup,
        )
    }
}
