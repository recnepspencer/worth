use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationReceipt, BridgeYieldedExecutionBasis,
};

use super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::workflow::WorthQueryWorkflowRunAffinity;
use super::{
    WorthQueryManagedRunCounters, WorthQueryPausedWorkflowGraphExecution,
    WorthQueryYieldTransitionCounters,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryFrozenWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::WorthQueryExecutionResourceAttemptEvidence;

#[cfg(test)]
#[path = "workflow_yield/corruption_tests.rs"]
mod corruption_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowYieldDenialKind {
    InstallationGenerationStale,
    YieldNotInstalled,
    CheckpointUnavailable,
    RetainedBytesExceeded,
}

pub struct WorthQueryWorkflowYieldDenied {
    pub(super) kind: WorthQueryWorkflowYieldDenialKind,
    pub(super) detail: Arc<str>,
    pub(super) paused: WorthQueryPausedWorkflowGraphExecution,
    pub(super) counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldDenied {
    pub const fn kind(&self) -> WorthQueryWorkflowYieldDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryYieldTransitionCounters {
        self.counters
    }

    pub fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        self.paused.active.running.counters()
    }

    pub fn into_paused(self) -> WorthQueryPausedWorkflowGraphExecution {
        self.paused
    }
}

#[must_use = "yield outcomes must be resolved into yielded, denied, or recovery authority"]
pub enum WorthQueryWorkflowYieldOutcome {
    Yielded(WorthQueryYieldedWorkflowRun),
    Denied(WorthQueryWorkflowYieldDenied),
    RecoveryRequired(super::WorthQueryWorkflowYieldRecoveryRequired),
}

#[must_use = "yielded workflow run retains exact cleanup or same-runtime readmission authority"]
pub struct WorthQueryYieldedWorkflowRun {
    affinity: WorthQueryWorkflowRunAffinity,
    relational_basis: WorthQueryManagedRelationalObservation,
    bridge: BridgeYieldedExecutionBasis,
    execution: WorthQueryRetainedManagedGraphExecution,
    artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    run_counters: WorthQueryManagedRunCounters,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    yield_counters: WorthQueryYieldTransitionCounters,
    inspection: super::WorthQueryYieldedWorkflowRunInspection,
}

pub(super) struct WorthQueryYieldedWorkflowCleanupAssociation {
    affinity: super::workflow::WorthQueryWorkflowYieldReleasePending,
    relational_basis: WorthQueryManagedRelationalObservation,
    bridge: BridgeExecutionBasisFinalizationReceipt,
    artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    checkpoint_release: crate::domain_computation::WorthQueryProviderCheckpointReleaseEvidence,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryYieldedWorkflowRun {
    pub(super) fn request_for_phase<'a>(
        &self,
        phase: &'a crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        worth_execution::ExecutionRequest<'a, 'a>,
        crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    > {
        phase.request_for_managed(&self.relational_basis)
    }

    pub(super) fn owner_from_yield_transition(
        minted: super::workflow_yield_transition::WorthQueryWorkflowYieldMintedOwner,
        _owner: super::workflow_yield_transition::WorthQueryWorkflowYieldMint,
    ) -> Self {
        let inspection = super::WorthQueryYieldedWorkflowRunInspection::capture(
            &minted.affinity,
            &minted.execution,
            &minted.run_counters,
            minted.yield_counters,
            minted.artifact_evidence,
        );
        Self {
            affinity: minted.affinity,
            relational_basis: minted.relational_basis,
            bridge: minted.bridge,
            execution: minted.execution,
            artifacts: minted.artifacts,
            artifact_evidence: minted.artifact_evidence,
            run_counters: minted.run_counters,
            provider_artifact_occurrences: minted.provider_artifact_occurrences,
            yield_counters: minted.yield_counters,
            inspection,
        }
    }

    pub(in crate::domain_computation::managed_run) fn owner_restore_from_readmission(
        restored: super::readmission::WorthQueryWorkflowYieldRestoredOwner,
        _owner: super::readmission::WorthQueryWorkflowReadmissionRestoreMint,
    ) -> Self {
        Self {
            affinity: restored.affinity,
            relational_basis: restored.relational_basis,
            bridge: restored.bridge,
            execution: restored.execution,
            artifacts: restored.artifacts,
            artifact_evidence: restored.artifact_evidence,
            run_counters: restored.run_counters,
            provider_artifact_occurrences: restored.provider_artifact_occurrences,
            yield_counters: restored.yield_counters,
            inspection: restored.inspection,
        }
    }

    pub(in crate::domain_computation::managed_run) fn readmission_stage_identity(
        &self,
    ) -> Option<&str> {
        self.execution.call.stage_identity()
    }

    pub(in crate::domain_computation::managed_run) fn readmission_stage_resources(
        &self,
        stage_identity: &str,
    ) -> Option<(
        Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )>{
        self.affinity
            .managed_stage_resources_and_evidence(stage_identity)
    }

    pub(in crate::domain_computation::managed_run) fn preflight_retained_provider_call(
        &self,
        stage_evidence: &WorthQueryExecutionResourceAttemptEvidence,
    ) -> Result<
        crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    >{
        self.affinity
            .preflight_retained_provider_call(&self.execution.call, stage_evidence)
    }

    pub(in crate::domain_computation::managed_run) fn query_readmission_denial(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> Option<(super::WorthQueryWorkflowReadmissionDenialKind, &'static str)> {
        if !self.affinity.belongs_to_runtime(runtime) {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::ForeignQueryRuntime,
                "yielded workflow belongs to a different Query execution runtime",
            ));
        }
        if !self.affinity.belongs_to_current_installation(runtime) {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::StaleInstallationGeneration,
                "yielded workflow belongs to a stale installed-operation generation",
            ));
        }
        if self.affinity.retained_capacity_reservation_count() == 0 {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::RetainedCapacityMismatch,
                "yielded workflow no longer owns its capacity-reservation package",
            ));
        }
        if !self.relational_basis.is_live() {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::RelationalLeaseNotLive,
                "yielded workflow Relational basis lease is no longer live",
            ));
        }
        if !self.execution.provider_generation_matches_anchor() {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::ProviderCheckpointMismatch,
                "workflow checkpoint generation no longer matches its provider anchor",
            ));
        }
        if !self.artifacts.registry_is_frozen_at_owned_generation()
            || self.artifacts.production_generation().ordinal()
                != self.artifact_evidence.production_generation()
        {
            return Some((
                super::WorthQueryWorkflowReadmissionDenialKind::ArtifactGenerationMismatch,
                "workflow artifact registry is not frozen at the yielded production generation",
            ));
        }
        None
    }

    pub(super) fn owner_into_cleanup_association(
        self,
        _owner: &super::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupPermit,
    ) -> WorthQueryYieldedWorkflowCleanupAssociation {
        let checkpoint_release = self.execution.release();
        WorthQueryYieldedWorkflowCleanupAssociation {
            affinity: self.affinity.finish_yield(),
            relational_basis: self.relational_basis,
            bridge: self.bridge.release(),
            artifacts: self.artifacts,
            checkpoint_release,
            artifact_evidence: self.artifact_evidence,
            run_counters: self.run_counters,
            yield_counters: self.yield_counters,
        }
    }

    pub(in crate::domain_computation::managed_run) fn owner_begin_readmission(
        self,
        owner: &super::readmission::WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> super::readmission::WorthQueryWorkflowYieldedAssociation {
        super::readmission::WorthQueryWorkflowYieldedAssociation::owner_admit_exact_yield(
            self.affinity,
            self.relational_basis,
            self.bridge,
            self.execution,
            self.artifacts,
            self.artifact_evidence,
            self.run_counters,
            self.provider_artifact_occurrences,
            self.yield_counters,
            self.inspection,
            owner,
        )
    }

    pub const fn inspection(&self) -> &super::WorthQueryYieldedWorkflowRunInspection {
        &self.inspection
    }

    #[must_use = "cleanup returns a workflow yielded-run cleanup outcome that must be resolved"]
    pub fn cleanup(self) -> super::WorthQueryWorkflowYieldCleanupOutcome {
        super::workflow_yield_cleanup::cleanup_yielded_workflow(self)
    }

    pub fn readmit_same_runtime(
        self,
        active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        query_runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> super::WorthQueryWorkflowReadmissionOutcome {
        super::readmission::readmit_workflow(active_request, self, query_runtime, bridge_runtime)
    }
}

impl WorthQueryYieldedWorkflowCleanupAssociation {
    #[allow(clippy::type_complexity)]
    pub(super) fn owner_into_parts(
        self,
        _owner: &super::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupPermit,
    ) -> (
        super::workflow::WorthQueryWorkflowYieldReleasePending,
        WorthQueryManagedRelationalObservation,
        BridgeExecutionBasisFinalizationReceipt,
        WorthQueryFrozenWorkflowArtifactAuthority,
        crate::domain_computation::WorthQueryProviderCheckpointReleaseEvidence,
        WorthQueryWorkflowArtifactRegistryEvidence,
        WorthQueryManagedRunCounters,
        WorthQueryYieldTransitionCounters,
    ) {
        (
            self.affinity,
            self.relational_basis,
            self.bridge,
            self.artifacts,
            self.checkpoint_release,
            self.artifact_evidence,
            self.run_counters,
            self.yield_counters,
        )
    }
}
