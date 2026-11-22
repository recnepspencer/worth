use std::sync::Arc;

use super::inspection::WorthQueryCompletedWorkflowRunCleanup;
use super::{WorthQueryWorkflowRunCleanupReceipt, WorthQueryWorkflowRunTerminal};
use crate::domain_computation::managed_run::bridge_terminal_disposition;
use crate::domain_computation::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCleanupFailureKind, WorthQueryManagedRunTerminalKind,
    WorthQueryWorkflowArtifactRegistryEvidence,
};

#[must_use = "workflow cleanup must resolve Complete, Pending, or RecoveryRequired"]
pub enum WorthQueryWorkflowRunCleanupOutcome {
    Complete(WorthQueryWorkflowRunCleanupReceipt),
    Pending(WorthQueryWorkflowRunCleanupPending),
    RecoveryRequired(WorthQueryWorkflowRunCleanupFailure),
}

impl std::fmt::Debug for WorthQueryWorkflowRunCleanupOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowRunCleanupOutcome")
            .field("disposition", &self.disposition())
            .finish()
    }
}

impl WorthQueryWorkflowRunCleanupOutcome {
    pub fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        match self {
            Self::Complete(receipt) => receipt.inspection().disposition(),
            Self::Pending(_) => WorthQueryManagedRunCleanupDisposition::CleanupPending,
            Self::RecoveryRequired(_) => WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
        }
    }
}

#[must_use = "workflow cleanup pending retains exact artifact and terminal retry authority"]
pub struct WorthQueryWorkflowRunCleanupPending {
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    provider_retained_bytes: usize,
    terminal: WorthQueryWorkflowRunTerminal,
}

impl WorthQueryWorkflowRunCleanupPending {
    pub fn pending_artifact_owner_count(&self) -> usize {
        self.artifact_evidence.retained_artifact_count()
    }

    pub fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }

    pub const fn provider_retained_bytes(&self) -> usize {
        self.provider_retained_bytes
    }

    #[must_use = "retry returns the next workflow cleanup outcome and must be resolved"]
    pub fn retry(self) -> WorthQueryWorkflowRunCleanupOutcome {
        finish(self.terminal)
    }
}

#[must_use = "workflow cleanup failure retains the exact terminal owner for retry"]
pub struct WorthQueryWorkflowRunCleanupFailure {
    failure_kind: WorthQueryManagedRunCleanupFailureKind,
    failure_detail: Arc<str>,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    terminal: WorthQueryWorkflowRunTerminal,
}

impl WorthQueryWorkflowRunCleanupFailure {
    pub fn failure_kind(&self) -> WorthQueryManagedRunCleanupFailureKind {
        self.failure_kind
    }

    pub fn failure_detail(&self) -> &str {
        &self.failure_detail
    }

    pub fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }

    #[must_use = "retry returns the next workflow cleanup outcome and must be resolved"]
    pub fn retry(self) -> WorthQueryWorkflowRunCleanupOutcome {
        finish(self.terminal)
    }
}

impl std::fmt::Debug for WorthQueryWorkflowRunCleanupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowRunCleanupFailure")
            .field("failure_kind", &self.failure_kind)
            .field("failure_detail", &self.failure_detail)
            .field("run_identity", &self.terminal.identity())
            .finish()
    }
}

pub(super) fn finish(
    mut terminal: WorthQueryWorkflowRunTerminal,
) -> WorthQueryWorkflowRunCleanupOutcome {
    let artifact_evidence = close_artifact_production(&terminal);
    if let Err(failure) = terminal
        .provider_cleanup
        .release_queue_occupancies(&mut terminal.bridge_basis, &mut terminal.provider_work)
    {
        return recovery_required(
            terminal,
            artifact_evidence,
            WorthQueryManagedRunCleanupFailureKind::QueueRelease(failure.kind()),
            failure.detail(),
        );
    }
    let provider_retained_bytes = terminal.provider_cleanup.reconcile_provider_memory();
    terminal
        .provider_work
        .reconcile_provider_retained_bytes(provider_retained_bytes);
    if cleanup_remains_pending(artifact_evidence, provider_retained_bytes) {
        return WorthQueryWorkflowRunCleanupOutcome::Pending(WorthQueryWorkflowRunCleanupPending {
            artifact_evidence,
            provider_retained_bytes,
            terminal,
        });
    }
    finalize(terminal, artifact_evidence)
}

fn close_artifact_production(
    terminal: &WorthQueryWorkflowRunTerminal,
) -> WorthQueryWorkflowArtifactRegistryEvidence {
    let registry = terminal.artifacts.registry();
    if terminal.kind == WorthQueryManagedRunTerminalKind::Completed {
        registry.close_released();
    } else {
        registry.close_cancelled();
    }
    registry.evidence()
}

fn cleanup_remains_pending(
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    provider_retained_bytes: usize,
) -> bool {
    artifact_evidence.retained_artifact_count() != 0
        || artifact_evidence.provider_release_pending_count() != 0
        || provider_retained_bytes != 0
}

fn finalize(
    terminal: WorthQueryWorkflowRunTerminal,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
) -> WorthQueryWorkflowRunCleanupOutcome {
    let WorthQueryWorkflowRunTerminal {
        affinity,
        kind,
        bridge_basis,
        relational_basis,
        artifacts,
        artifact_evidence_at_terminal,
        counters,
        provider_work,
        provider_cleanup,
    } = terminal;
    let bridge = match bridge_basis.finalize(bridge_terminal_disposition(kind)) {
        Ok(receipt) => receipt,
        Err(failure) => {
            let failure_kind = failure.kind();
            let failure_detail = failure.detail().to_owned();
            return recovery_required(
                WorthQueryWorkflowRunTerminal {
                    affinity,
                    kind,
                    bridge_basis: failure.into_basis(),
                    relational_basis,
                    artifacts,
                    artifact_evidence_at_terminal,
                    counters,
                    provider_work,
                    provider_cleanup,
                },
                artifact_evidence,
                WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(failure_kind),
                &failure_detail,
            );
        }
    };
    drop(artifacts);
    let disposition = closed_disposition(&provider_work, artifact_evidence);
    let logical_run_identity = Arc::from(affinity.logical_identity());
    let identity = Arc::from(affinity.attempt_identity());
    let receipt = WorthQueryWorkflowRunCleanupReceipt::from_completed(
        WorthQueryCompletedWorkflowRunCleanup {
            logical_run_identity,
            identity,
            terminal: kind,
            disposition,
            bridge,
            relational: relational_basis.release(),
            attempt: affinity.release(),
            artifact_evidence,
            counters,
            provider_work,
        },
    );
    WorthQueryWorkflowRunCleanupOutcome::Complete(receipt)
}

fn closed_disposition(
    provider_work: &WorthQueryManagedProviderWorkEvidence,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
) -> WorthQueryManagedRunCleanupDisposition {
    if provider_work.requires_cleanup_recovery()
        || artifact_evidence.provider_release_recovery_required_count() != 0
    {
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    } else {
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    }
}

fn recovery_required(
    terminal: WorthQueryWorkflowRunTerminal,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    failure_kind: WorthQueryManagedRunCleanupFailureKind,
    failure_detail: &str,
) -> WorthQueryWorkflowRunCleanupOutcome {
    WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(WorthQueryWorkflowRunCleanupFailure {
        failure_kind,
        failure_detail: Arc::from(failure_detail),
        artifact_evidence,
        terminal,
    })
}
