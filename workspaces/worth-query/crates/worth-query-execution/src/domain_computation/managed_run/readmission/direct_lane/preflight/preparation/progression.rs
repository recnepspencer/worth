use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionCleanupOutcome, BridgeExecutionBasisReadmissionRecoveryRequired,
    RuntimeBridge,
};

use worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionPending;

use super::{
    WorthQueryDirectBridgeReadmissionAttempt, WorthQueryDirectBridgeReadmissionPending,
    WorthQueryDirectRetainedState, WorthQueryDirectYieldedParts, WorthQueryDirectYieldedState,
};
use crate::domain_computation::managed_run::managed_graph_execution::WorthQueryManagedGraphExecution;
use crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestorePending;
use crate::domain_computation::managed_run::provider_restore::{
    WorthQueryManagedGraphRestoreCommitOutcome, WorthQueryManagedGraphRestoreRecoveryKind,
};
use crate::domain_computation::managed_run::readmission::direct_outcome::{
    WorthQueryDirectReadmissionDenialKind, WorthQueryDirectReadmissionDenied,
    WorthQueryDirectReadmissionOutcome,
};
use crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress;
use crate::domain_computation::managed_run::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use crate::domain_computation::managed_run::run_affinity::{
    WorthQueryDirectRunProviderRestoreOutcome, WorthQueryDirectRunReadmissionPending,
};
use crate::domain_computation::managed_run::{
    WorthQueryActiveDirectGraphExecution, WorthQueryRunningDirectRun, WorthQueryYieldedDirectRun,
};
use crate::domain_computation::WorthQueryExecutionRuntime;

pub(super) mod recovery;

#[cfg(test)]
mod tests;

pub use recovery::{
    WorthQueryDirectReadmissionCleanupInspection, WorthQueryDirectReadmissionCleanupOutcome,
    WorthQueryDirectReadmissionCleanupPending, WorthQueryDirectReadmissionCleanupPendingInspection,
    WorthQueryDirectReadmissionCleanupReceipt, WorthQueryDirectReadmissionCleanupRequired,
    WorthQueryDirectReadmissionRecoveryKind, WorthQueryDirectReadmissionRecoveryPosture,
    WorthQueryDirectReadmissionRecoveryRequired, WorthQueryDirectReadmissionTerminalRecovery,
    WorthQueryDirectReadmissionYieldReassembled, WorthQueryDirectReadmissionYieldReassemblyOutcome,
    WorthQueryDirectReadmissionYieldReassemblyRecovery,
};

use recovery::{WorthQueryDirectBridgeCleanupRecoveryState, WorthQueryDirectProviderRecoveryState};

struct WorthQueryDirectProviderRestorePending {
    state: WorthQueryDirectRetainedState,
    provider: WorthQueryManagedGraphRestorePending,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

pub(super) struct WorthQueryDirectCommitReady {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

struct WorthQueryDirectRollbackPending {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryRetainedManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

struct WorthQueryDirectProviderRecoverySeed {
    state: WorthQueryDirectRetainedState,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

struct WorthQueryDirectRollbackAfterBridgeAbort {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryRetainedManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
}

pub(super) fn bridge_cleanup_recovery_required(
    detail: impl Into<Arc<str>>,
    progress: WorthQueryReadmissionProgress,
    attempt: WorthQueryDirectBridgeReadmissionAttempt,
    bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
) -> WorthQueryDirectReadmissionRecoveryRequired {
    WorthQueryDirectReadmissionRecoveryRequired::bridge_cleanup(
        detail,
        progress,
        WorthQueryDirectBridgeCleanupRecoveryState::from_bridge_attempt(attempt, bridge),
    )
}

pub(in crate::domain_computation::managed_run) fn readmit_direct(
    active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedDirectRun,
    query_runtime: &WorthQueryExecutionRuntime,
    bridge_runtime: &RuntimeBridge,
) -> WorthQueryDirectReadmissionOutcome {
    let active_request = match yielded.request_for_phase(active_request) {
        Ok(request) => request,
        Err(_) => {
            return WorthQueryDirectReadmissionOutcome::Denied(
                WorthQueryDirectReadmissionDenied::new(
                    WorthQueryDirectReadmissionDenialKind::ForeignAdvancementPhase,
                    "the phase belongs to another installed runtime",
                    yielded,
                    WorthQueryReadmissionProgress::default().evidence(),
                ),
            )
        }
    };
    let (pending, counters) =
        match super::prepare_direct_provider_restore(yielded, query_runtime, bridge_runtime) {
            Ok(prepared) => prepared,
            Err(outcome) => return outcome,
        };
    restore_direct(active_request, pending, bridge_runtime, counters)
}

pub(super) fn restore_direct(
    active_request: worth_execution::ExecutionRequest<'_, '_>,

    pending: WorthQueryDirectBridgeReadmissionPending,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionOutcome {
    let WorthQueryDirectBridgeReadmissionPending {
        state,
        execution,
        resource: resource_pending,
        bridge: bridge_pending,
        contract,
    } = pending;
    progress.attempted_provider_restore();
    let provider = match resource_pending.restore_provider(
        active_request,
        execution,
        contract,
        &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(
        ),
    ) {
        WorthQueryDirectRunProviderRestoreOutcome::Pending { resource, provider } => {
            WorthQueryDirectProviderRestorePending {
                state,
                provider,
                resource,
                bridge: bridge_pending,
            }
        }
        WorthQueryDirectRunProviderRestoreOutcome::Denied { resource, denial } => {
            let detail = Arc::from(denial.detail());
            return abort_to_denial(
                WorthQueryDirectReadmissionDenialKind::ProviderRestoreDenied,
                detail,
                WorthQueryDirectRollbackPending {
                    state,
                    execution: denial.into_retained(),
                    resource,
                    bridge: bridge_pending,
                },
                progress,
            );
        }
        WorthQueryDirectRunProviderRestoreOutcome::RecoveryRequired { resource, recovery } => {
            let kind = recovery_kind(recovery.kind());
            let detail = Arc::from(recovery.detail());
            return WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                WorthQueryDirectReadmissionRecoveryRequired::provider(
                    kind,
                    detail,
                    progress,
                    WorthQueryDirectProviderRecoveryState::from_seed(
                        WorthQueryDirectProviderRecoverySeed {
                            state,
                            resource,
                            bridge: bridge_pending,
                        },
                        recovery,
                    ),
                ),
            );
        }
    };
    commit_provider_restore(provider, bridge_runtime, progress)
}

fn commit_provider_restore(
    pending: WorthQueryDirectProviderRestorePending,
    bridge_runtime: &RuntimeBridge,
    progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionOutcome {
    let WorthQueryDirectProviderRestorePending {
        state,
        provider,
        resource,
        bridge,
    } = pending;
    let recovery_seed = WorthQueryDirectProviderRecoverySeed {
        state,
        resource,
        bridge,
    };
    let execution = match provider.commit(None) {
        WorthQueryManagedGraphRestoreCommitOutcome::Restored(execution) => execution,
        WorthQueryManagedGraphRestoreCommitOutcome::RecoveryRequired(recovery) => {
            let kind = recovery_kind(recovery.kind());
            let detail = Arc::from(recovery.detail());
            return WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                WorthQueryDirectReadmissionRecoveryRequired::provider(
                    kind,
                    detail,
                    progress,
                    WorthQueryDirectProviderRecoveryState::from_seed(recovery_seed, recovery),
                ),
            );
        }
    };
    commit_direct(
        WorthQueryDirectCommitReady {
            state: recovery_seed.state,
            execution,
            resource: recovery_seed.resource,
            bridge: recovery_seed.bridge,
        },
        bridge_runtime,
        progress,
    )
}

fn commit_direct(
    pending: WorthQueryDirectCommitReady,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionOutcome {
    let WorthQueryDirectCommitReady {
        state,
        execution,
        resource,
        bridge,
    } = pending;
    let (bridge_basis, bridge_counters) = bridge_runtime
        .commit_yielded_execution_basis_readmission(bridge)
        .into_parts();
    progress.observe_bridge(bridge_counters);
    let affinity = resource.commit(
        crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
    );
    progress.committed_attempt();
    let active = WorthQueryActiveDirectGraphExecution::new(
        WorthQueryRunningDirectRun {
            affinity,
            bridge_basis,
            relational_basis: state.relational_basis,
            counters: state.run_counters,
        },
        execution,
    );
    WorthQueryDirectReadmissionOutcome::Readmitted(
        crate::domain_computation::managed_run::WorthQueryReadmittedDirectGraphExecution::new(
            active,
            progress.evidence(),
        ),
    )
}

fn abort_to_denial(
    kind: WorthQueryDirectReadmissionDenialKind,
    detail: impl Into<Arc<str>>,
    pending: WorthQueryDirectRollbackPending,
    mut progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionOutcome {
    let detail = detail.into();
    let WorthQueryDirectRollbackPending {
        state,
        execution,
        resource,
        bridge,
    } = pending;
    let rollback = WorthQueryDirectRollbackAfterBridgeAbort {
        state,
        execution,
        resource,
    };
    match bridge.abort() {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => {
            let (bridge, bridge_counters) = returned.into_parts();
            progress.observe_bridge(bridge_counters);
            denied(
                kind,
                detail,
                WorthQueryDirectYieldedParts {
                    state: WorthQueryDirectYieldedState {
                        affinity: rollback.resource.abort(
                            crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
                        ),
                        retained: rollback.state,
                    },
                    bridge,
                    execution: rollback.execution,
                }
                .into_yielded(),
                progress,
            )
        }
        BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(recovery) => {
            progress.observe_bridge(recovery.counters());
            WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                WorthQueryDirectReadmissionRecoveryRequired::bridge_cleanup(
                    format!("{detail}; Bridge cleanup failed: {}", recovery.detail()),
                    progress,
                    WorthQueryDirectBridgeCleanupRecoveryState::from_rollback(rollback, recovery),
                ),
            )
        }
    }
}

fn denied(
    kind: WorthQueryDirectReadmissionDenialKind,
    detail: impl Into<Arc<str>>,
    yielded: WorthQueryYieldedDirectRun,
    progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionOutcome {
    WorthQueryDirectReadmissionOutcome::Denied(WorthQueryDirectReadmissionDenied::new(
        kind,
        detail,
        yielded,
        progress.evidence(),
    ))
}

fn recovery_kind(
    kind: WorthQueryManagedGraphRestoreRecoveryKind,
) -> WorthQueryDirectReadmissionRecoveryKind {
    match kind {
        WorthQueryManagedGraphRestoreRecoveryKind::ProviderRestorePanicked => {
            WorthQueryDirectReadmissionRecoveryKind::ProviderRestorePanicked
        }
        WorthQueryManagedGraphRestoreRecoveryKind::
            ProviderRestoreRejectedAfterExecutionAdmission => {
                WorthQueryDirectReadmissionRecoveryKind::
                    ProviderRestoreRejectedAfterExecutionAdmission
            }
        WorthQueryManagedGraphRestoreRecoveryKind::RestoredExecutionReleaseRecoveryRequired => {
            WorthQueryDirectReadmissionRecoveryKind::RestoredExecutionReleaseRecoveryRequired
        }
        WorthQueryManagedGraphRestoreRecoveryKind::CheckpointReleasePanicked => {
            WorthQueryDirectReadmissionRecoveryKind::CheckpointReleasePanicked
        }
    }
}
