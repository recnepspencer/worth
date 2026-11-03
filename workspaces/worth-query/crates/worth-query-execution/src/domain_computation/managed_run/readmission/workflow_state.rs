use std::sync::Arc;

use crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionPending, BridgeExecutionBasisReadmissionRecoveryRequired,
    BridgeYieldedExecutionBasis, BridgeYieldedExecutionBasisPreflight,
};

use super::super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::super::super::workflow::{
    WorthQueryWorkflowRunAffinity, WorthQueryWorkflowRunReadmissionPending,
    WorthQueryWorkflowRunRestoredPending, WorthQueryWorkflowRunTransitionPermit,
};
use super::super::super::{
    WorthQueryManagedRunCounters, WorthQueryYieldTransitionCounters, WorthQueryYieldedWorkflowRun,
};
use super::super::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryPermit;
use super::WorthQueryWorkflowReadmissionProgressionPermit;
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryArtifactProductionGenerationCommitted,
    WorthQueryFrozenWorkflowArtifactAuthority, WorthQueryWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistryEvidence,
};

pub(super) struct WorthQueryWorkflowYieldedState {
    relational_basis: WorthQueryManagedRelationalObservation,
    artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    run_counters: WorthQueryManagedRunCounters,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    yield_counters: WorthQueryYieldTransitionCounters,
    inspection: crate::domain_computation::WorthQueryYieldedWorkflowRunInspection,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowYieldedAssociation {
    state: WorthQueryWorkflowYieldedState,
    affinity: WorthQueryWorkflowRunAffinity,
    bridge: BridgeYieldedExecutionBasis,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(super) struct WorthQueryWorkflowPreflightAssociation {
    state: WorthQueryWorkflowYieldedState,
    affinity: WorthQueryWorkflowRunAffinity,
    bridge: BridgeYieldedExecutionBasisPreflight,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(super) struct WorthQueryWorkflowProvisionalAssociation {
    state: WorthQueryWorkflowYieldedState,
    resource: WorthQueryWorkflowRunReadmissionPending,
    bridge: BridgeYieldedExecutionBasisPreflight,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(in crate::domain_computation::managed_run::readmission) struct WorthQueryWorkflowBridgePendingAssociation
{
    state: WorthQueryWorkflowYieldedState,
    resource: WorthQueryWorkflowRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(in crate::domain_computation::managed_run::readmission) struct WorthQueryWorkflowRestoredAssociation
{
    state: WorthQueryWorkflowYieldedState,
    resource: WorthQueryWorkflowRunRestoredPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

pub(in crate::domain_computation::managed_run::readmission) struct WorthQueryWorkflowBridgeRecoveryAssociation
{
    state: WorthQueryWorkflowYieldedState,
    affinity: WorthQueryWorkflowRunAffinity,
    bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowReadmissionCommitState {
    relational_basis: WorthQueryManagedRelationalObservation,
    artifacts: WorthQueryWorkflowArtifactAuthority,
    run_counters: WorthQueryManagedRunCounters,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowReadmissionRestoreMint {
    _owner: (),
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowYieldRestoredOwner {
    pub(in crate::domain_computation::managed_run) affinity: WorthQueryWorkflowRunAffinity,
    pub(in crate::domain_computation::managed_run) relational_basis:
        WorthQueryManagedRelationalObservation,
    pub(in crate::domain_computation::managed_run) bridge: BridgeYieldedExecutionBasis,
    pub(in crate::domain_computation::managed_run) execution:
        WorthQueryRetainedManagedGraphExecution,
    pub(in crate::domain_computation::managed_run) artifacts:
        WorthQueryFrozenWorkflowArtifactAuthority,
    pub(in crate::domain_computation::managed_run) artifact_evidence:
        WorthQueryWorkflowArtifactRegistryEvidence,
    pub(in crate::domain_computation::managed_run) run_counters: WorthQueryManagedRunCounters,
    pub(in crate::domain_computation::managed_run) provider_artifact_occurrences:
        Arc<WorthQueryArtifactOccurrenceLedger>,
    pub(in crate::domain_computation::managed_run) yield_counters:
        WorthQueryYieldTransitionCounters,
    pub(in crate::domain_computation::managed_run) inspection:
        crate::domain_computation::WorthQueryYieldedWorkflowRunInspection,
}

impl WorthQueryWorkflowReadmissionRestoreMint {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

#[path = "workflow_state/bridge_progression.rs"]
mod bridge_progression;
#[path = "workflow_state/commit_progression.rs"]
mod commit_progression;
#[path = "workflow_state/preflight_progression.rs"]
mod preflight_progression;
#[path = "workflow_state/recovery_progression.rs"]
mod recovery_progression;
#[path = "recovery/workflow_cleanup.rs"]
pub(in crate::domain_computation::managed_run::readmission) mod workflow_cleanup_owner;

pub(in crate::domain_computation::managed_run::readmission) use commit_progression::WorthQueryWorkflowCommittedAssociation;

impl WorthQueryWorkflowYieldedState {
    fn restore_yielded(
        self,
        affinity: WorthQueryWorkflowRunAffinity,
        bridge: BridgeYieldedExecutionBasis,
        execution: WorthQueryRetainedManagedGraphExecution,
    ) -> WorthQueryYieldedWorkflowRun {
        WorthQueryYieldedWorkflowRun::owner_restore_from_readmission(
            WorthQueryWorkflowYieldRestoredOwner {
                affinity,
                relational_basis: self.relational_basis,
                bridge,
                execution,
                artifacts: self.artifacts,
                artifact_evidence: self.artifact_evidence,
                run_counters: self.run_counters,
                provider_artifact_occurrences: self.provider_artifact_occurrences,
                yield_counters: self.yield_counters,
                inspection: self.inspection,
            },
            WorthQueryWorkflowReadmissionRestoreMint::mint(),
        )
    }

    fn commit_artifact_generation(
        self,
        committed: WorthQueryArtifactProductionGenerationCommitted,
    ) -> WorthQueryWorkflowReadmissionCommitState {
        WorthQueryWorkflowReadmissionCommitState {
            relational_basis: self.relational_basis,
            artifacts: self.artifacts.activate_after_readmission(committed),
            run_counters: self.run_counters,
            provider_artifact_occurrences: self.provider_artifact_occurrences,
        }
    }
}

impl WorthQueryWorkflowYieldedAssociation {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::managed_run) fn owner_admit_exact_yield(
        affinity: WorthQueryWorkflowRunAffinity,
        relational_basis: WorthQueryManagedRelationalObservation,
        bridge: BridgeYieldedExecutionBasis,
        execution: WorthQueryRetainedManagedGraphExecution,
        artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
        artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
        run_counters: WorthQueryManagedRunCounters,
        provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
        yield_counters: WorthQueryYieldTransitionCounters,
        inspection: crate::domain_computation::WorthQueryYieldedWorkflowRunInspection,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self {
            state: WorthQueryWorkflowYieldedState {
                relational_basis,
                artifacts,
                artifact_evidence,
                run_counters,
                provider_artifact_occurrences,
                yield_counters,
                inspection,
            },
            affinity,
            bridge,
            execution,
        }
    }
}

impl WorthQueryWorkflowReadmissionCommitState {
    pub(in crate::domain_computation::managed_run) fn owner_install(
        self,
        affinity: WorthQueryWorkflowRunAffinity,
        bridge_basis: worth_runtime_bridge::facade::BridgeBoundExecutionBasis,
        _owner: &WorthQueryWorkflowRunTransitionPermit,
    ) -> super::super::super::WorthQueryRunningWorkflowRun {
        super::super::super::WorthQueryRunningWorkflowRun::owner_restore_readmission(
            affinity,
            bridge_basis,
            self.relational_basis,
            self.run_counters,
            self.artifacts,
            self.provider_artifact_occurrences,
            _owner,
        )
    }
}
