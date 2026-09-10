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

pub(in crate::domain_computation::managed_run) use cleanup::{
    WorthQueryWorkflowAffinityCleanupReceipt, WorthQueryWorkflowYieldReleasePending,
};
pub(in crate::domain_computation::managed_run::workflow) use terminal::WorthQueryWorkflowRunTerminalAffinity;

pub(in crate::domain_computation) struct WorthQueryWorkflowRunTransitionPermit {
    _owner: (),
}

impl WorthQueryWorkflowRunTransitionPermit {
    const fn mint() -> Self {
        Self { _owner: () }
    }
}

/// Move-only association between one logical workflow run, its current live
/// resource attempt, and the provider-work ledger bound to that attempt.
pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowRunAffinity {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowExecutionResourceAttempt,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowRunReadmissionPending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowResourceReadmissionPreProvider,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowRunRestoredPending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowResourceReadmissionPostProvider,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub(in crate::domain_computation::managed_run) enum WorthQueryWorkflowRunProviderRestoreOutcome {
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
    pub(in crate::domain_computation::managed_run) fn provider_session_matches_attempt(
        attempt: &WorthQueryWorkflowExecutionResourceAttempt,
    ) -> bool {
        attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
            .attempt_identity()
            == attempt.attempt_identity().as_str()
    }

    pub(in crate::domain_computation::managed_run) fn initial(
        attempt: WorthQueryWorkflowExecutionResourceAttempt,
    ) -> Self {
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

    pub(in crate::domain_computation::managed_run) fn release_initial(
        self,
    ) -> crate::domain_computation::WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        self.attempt.release()
    }

    pub(in crate::domain_computation::managed_run) fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub(in crate::domain_computation::managed_run) fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub(in crate::domain_computation::managed_run) fn attempt_description_arc(&self) -> &Arc<str> {
        self.attempt.attempt_identity().description_arc()
    }

    pub(in crate::domain_computation::managed_run) fn belongs_to_operation(
        &self,
        operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        self.attempt.binding_authority().binding_identity() == operation.binding_identity()
    }

    pub(in crate::domain_computation::managed_run) fn retained_capacity_reservation_count(
        &self,
    ) -> usize {
        self.attempt.retained_capacity_reservation_count()
    }

    pub(in crate::domain_computation::managed_run) fn operation_resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan
    {
        self.attempt.operation_resources()
    }

    pub(in crate::domain_computation::managed_run) fn resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan
    {
        self.attempt.resources()
    }

    pub(in crate::domain_computation::managed_run) fn evidence(
        &self,
    ) -> &WorthQueryExecutionResourceAttemptEvidence {
        self.attempt.evidence()
    }

    pub(super) fn provider_work_mut(&mut self) -> &mut WorthQueryManagedProviderWorkLedger {
        &mut self.provider_work
    }

    pub(in crate::domain_computation::managed_run) fn record_provider_execution_release(
        &mut self,
        evidence: &crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence,
    ) {
        self.provider_work
            .record_provider_execution_release(evidence);
    }

    pub(in crate::domain_computation::managed_run) fn abandon_provider_work(&mut self) {
        self.provider_work.abandon();
    }

    pub(in crate::domain_computation::managed_run) fn interrupt_provider_step_call(&mut self) {
        self.provider_work.interrupt_step_call();
    }

    pub(in crate::domain_computation::managed_run) fn settle_provider_artifacts(
        &mut self,
        evidence: crate::domain_computation::artifact_owner::WorthQueryArtifactOccurrenceSnapshot,
    ) {
        self.provider_work.settle_artifacts(evidence);
    }

    pub(in crate::domain_computation::managed_run) fn provider_work_snapshot(
        &self,
    ) -> WorthQueryManagedProviderWorkEvidence {
        self.provider_work.snapshot()
    }

    pub(in crate::domain_computation::managed_run) fn provider_work_has_uncertainty(&self) -> bool {
        self.provider_work.has_uncertainty()
    }

    pub(in crate::domain_computation::managed_run) fn provider_session_description(&self) -> &str {
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

    pub(in crate::domain_computation::managed_run) fn binding_identity_projection(&self) -> &str {
        self.attempt.binding_authority().binding_identity()
    }

    pub(in crate::domain_computation::managed_run) fn operation_identity_projection(&self) -> &str {
        self.attempt.binding_authority().operation_identity()
    }

    pub(in crate::domain_computation::managed_run) fn basis_identity_projection(&self) -> &str {
        self.attempt.binding_authority().basis_identity()
    }

    pub(in crate::domain_computation::managed_run) fn installation_generation_projection(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstallationGeneration {
        self.attempt.binding_authority().installation_generation()
    }

    pub(in crate::domain_computation::managed_run) fn belongs_to_runtime(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt.binding_authority().belongs_to(runtime)
    }

    pub(in crate::domain_computation::managed_run) fn belongs_to_current_installation(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt
            .binding_authority()
            .belongs_to_current_installation(runtime)
    }

    pub(in crate::domain_computation::managed_run) fn preflight_retained_provider_call(
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

    pub(in crate::domain_computation::managed_run) fn bind_graph_commit_call(
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

    pub(in crate::domain_computation::managed_run) fn completed_evidence_authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub(in crate::domain_computation::managed_run) fn completed_evidence_session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.attempt
            .provider_session_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
    }

    pub(in crate::domain_computation::managed_run) fn bind_managed_workflow_artifacts(
        &self,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactAuthority,
        crate::domain_computation::WorthQueryArtifactDenial,
    > {
        self.attempt
            .bind_workflow_artifacts_for_managed_run(&WorthQueryWorkflowRunTransitionPermit::mint())
    }

    pub(in crate::domain_computation::managed_run) fn binding_authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub(in crate::domain_computation::managed_run) fn managed_stage_resources_and_evidence(
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

    pub(in crate::domain_computation::managed_run) fn begin_readmission(
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

    pub(in crate::domain_computation::managed_run::workflow) fn into_terminal_parts(
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
