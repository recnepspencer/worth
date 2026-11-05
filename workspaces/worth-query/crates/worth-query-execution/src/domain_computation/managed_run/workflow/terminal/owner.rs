use super::super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::super::run_affinity::WorthQueryWorkflowRunTerminalAffinity;
use super::super::WorthQueryRunningWorkflowRun;
use super::{cleanup, WorthQueryWorkflowRunCleanupOutcome};
use crate::domain_computation::artifact_owner::{
    WorthQueryWorkflowArtifactAuthority, WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::managed_run::provider_work::WorthQueryManagedProviderCleanupAuthority;
use {crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence, crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters, crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind};

#[must_use = "a workflow run terminal retains resources that must be cleaned up"]
pub struct WorthQueryWorkflowRunTerminal {
    pub(super) affinity: WorthQueryWorkflowRunTerminalAffinity,
    pub(super) kind: WorthQueryManagedRunTerminalKind,
    pub(super) bridge_basis: BridgeBoundExecutionBasis,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) artifacts: WorthQueryWorkflowArtifactAuthority,
    pub(super) artifact_evidence_at_terminal: WorthQueryWorkflowArtifactRegistryEvidence,
    pub(super) counters: WorthQueryManagedRunCounters,
    pub(super) provider_work: WorthQueryManagedProviderWorkEvidence,
    pub(super) provider_cleanup: WorthQueryManagedProviderCleanupAuthority,
}

impl WorthQueryWorkflowRunTerminal {
    pub(in crate::domain_computation::managed_run::workflow) fn from_running(
        mut running: WorthQueryRunningWorkflowRun,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> Self {
        let artifact_evidence_at_terminal = running.artifacts.registry().freeze_production();
        running
            .affinity
            .provider_work_mut()
            .settle_artifacts(running.provider_artifact_occurrences.snapshot());
        let (affinity, provider_work, provider_cleanup) = running.affinity.into_terminal_parts();
        Self {
            affinity,
            kind,
            bridge_basis: running.bridge_basis,
            relational_basis: running.relational_basis,
            artifacts: running.artifacts,
            artifact_evidence_at_terminal,
            counters: running.counters,
            provider_work,
            provider_cleanup,
        }
    }

    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn kind(&self) -> WorthQueryManagedRunTerminalKind {
        self.kind
    }

    pub fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }

    pub fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }

    pub fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence_at_terminal
    }

    #[must_use = "workflow cleanup returns authority that must be resolved"]
    pub fn cleanup(self) -> WorthQueryWorkflowRunCleanupOutcome {
        cleanup::finish(self)
    }
}
