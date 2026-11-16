use std::sync::Arc;

use crate::domain_computation::managed_run::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionCleanupOutcome, BridgeExecutionBasisReadmissionPending,
    BridgeExecutionBasisReadmissionRecoveryRequired,
};

use super::WorthQueryDirectBridgeCleanupRecoveryState;
use crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreCleanupRequired;
use crate::domain_computation::managed_run::readmission::direct_lane::preflight::preparation::WorthQueryDirectRetainedState;
use crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress;
use crate::domain_computation::managed_run::run_affinity::WorthQueryDirectRunAffinity;
use crate::domain_computation::managed_run::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryYieldTransitionCounters,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::{
    WorthQueryDirectExecutionAttemptReleaseReceipt, WorthQueryProviderCheckpointReleaseEvidence,
};

mod inspection;
use crate::domain_computation::managed_run::durable_custody::ManagedRunCustody;
use crate::domain_computation::primary_graph::query_obligations::{
    QueryManagedExecutionRelease, QueryManagedRunPosture, QueryManagedRunResources,
};

use inspection::WorthQueryCompletedDirectReadmissionCleanup;
pub use inspection::{
    WorthQueryDirectReadmissionCleanupInspection,
    WorthQueryDirectReadmissionCleanupPendingInspection,
};

#[must_use = "direct readmission cleanup retains its receipt or unfinished cleanup authority"]
pub enum WorthQueryDirectReadmissionCleanupOutcome {
    Complete(WorthQueryDirectReadmissionCleanupReceipt),
    Pending(WorthQueryDirectReadmissionCleanupPending),
    RecoveryRequired(WorthQueryDirectReadmissionCleanupReceipt),
}

#[must_use = "direct readmission cleanup must be finished"]
pub struct WorthQueryDirectReadmissionCleanupRequired {
    state: WorthQueryDirectRetainedState,
    affinity: WorthQueryDirectRunAffinity,
    bridge: WorthQueryDirectReadmissionBridgeCleanup,
    provider: WorthQueryManagedGraphRestoreCleanupRequired,
    progress: WorthQueryReadmissionProgress,
}

#[must_use = "pending direct readmission cleanup retains Bridge cleanup authority"]
pub struct WorthQueryDirectReadmissionCleanupPending {
    receipt: WorthQueryDirectReadmissionPartialCleanupReceipt,
    bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
    inspection: WorthQueryDirectReadmissionCleanupPendingInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDirectReadmissionCleanupReceipt {
    inspection: WorthQueryDirectReadmissionCleanupInspection,
}

struct WorthQueryDirectReadmissionPartialCleanupReceipt {
    custody: Option<Arc<ManagedRunCustody>>,
    resources: QueryManagedRunResources,
    logical_run_identity: Arc<str>,
    yielded_attempt_identity: Arc<str>,
    checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
    relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    attempt: WorthQueryDirectExecutionAttemptReleaseReceipt,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
    readmission_progress: WorthQueryReadmissionProgress,
}

enum WorthQueryDirectReadmissionBridgeCleanup {
    Pending(BridgeExecutionBasisReadmissionPending),
    Recovery(BridgeExecutionBasisReadmissionRecoveryRequired),
}

impl WorthQueryDirectReadmissionCleanupRequired {
    pub(super) fn provider(
        state: WorthQueryDirectRetainedState,
        affinity: WorthQueryDirectRunAffinity,
        bridge: BridgeExecutionBasisReadmissionPending,
        provider: WorthQueryManagedGraphRestoreCleanupRequired,
        progress: WorthQueryReadmissionProgress,
    ) -> Self {
        Self {
            state,
            affinity,
            bridge: WorthQueryDirectReadmissionBridgeCleanup::Pending(bridge),
            provider,
            progress,
        }
    }

    pub(super) fn bridge_recovery(
        recovery: WorthQueryDirectBridgeCleanupRecoveryState,
        progress: WorthQueryReadmissionProgress,
    ) -> Self {
        Self {
            state: recovery.state,
            affinity: recovery.affinity,
            bridge: WorthQueryDirectReadmissionBridgeCleanup::Recovery(recovery.bridge),
            provider: WorthQueryManagedGraphRestoreCleanupRequired::retained(
                recovery.execution,
                None,
            ),
            progress,
        }
    }

    #[must_use = "finishing direct readmission cleanup returns the exact cleanup posture"]
    pub fn finish(mut self) -> WorthQueryDirectReadmissionCleanupOutcome {
        let custody = self.affinity.custody();
        let mut resources =
            QueryManagedRunResources::observe(false, &self.affinity.provider_work_snapshot());
        let provider = self.provider.finish();
        self.affinity
            .provider_work_mut()
            .retain_provider_memory(provider.memory);
        resources.checkpoint = Some(provider.checkpoint_release.disposition());
        resources.restored_execution = provider
            .restored_execution_release
            .as_ref()
            .map(QueryManagedExecutionRelease::observe);
        if let Some(release) = &provider.restored_execution_release {
            self.affinity
                .provider_work_mut()
                .record_provider_execution_release(release);
        }
        let (affinity, mut provider_work, mut provider_cleanup) =
            self.affinity.into_terminal_parts();
        provider_work
            .reconcile_provider_retained_bytes(provider_cleanup.reconcile_provider_memory());
        let (logical_run_identity, yielded_attempt_identity) = affinity.terminal_descriptions();
        let partial = WorthQueryDirectReadmissionPartialCleanupReceipt {
            custody,
            resources,
            logical_run_identity,
            yielded_attempt_identity,
            checkpoint_release: provider.checkpoint_release,
            restored_execution_release: provider.restored_execution_release,
            relational: self.state.relational_basis.release(),
            attempt: affinity.release(),
            run_counters: self.state.run_counters,
            provider_work,
            yield_counters: self.state.yield_counters,
            readmission_progress: self.progress,
        };
        finish_bridge_cleanup(
            partial,
            match self.bridge {
                WorthQueryDirectReadmissionBridgeCleanup::Pending(bridge) => bridge.abort(),
                WorthQueryDirectReadmissionBridgeCleanup::Recovery(bridge) => {
                    bridge.retry_cleanup()
                }
            },
        )
    }
}

impl WorthQueryDirectReadmissionCleanupPending {
    #[must_use = "retrying direct readmission cleanup returns the exact cleanup posture"]
    pub fn retry(self) -> WorthQueryDirectReadmissionCleanupOutcome {
        finish_bridge_cleanup(self.receipt, self.bridge.retry_cleanup())
    }

    pub const fn inspection(&self) -> &WorthQueryDirectReadmissionCleanupPendingInspection {
        &self.inspection
    }
}

fn finish_bridge_cleanup(
    mut partial: WorthQueryDirectReadmissionPartialCleanupReceipt,
    bridge: BridgeExecutionBasisReadmissionCleanupOutcome,
) -> WorthQueryDirectReadmissionCleanupOutcome {
    match bridge {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => {
            let (bridge, bridge_counters) = returned.into_parts();
            partial.readmission_progress.observe_bridge(bridge_counters);
            let recovery_required = direct_cleanup_recovery_required(&partial);
            let disposition = if recovery_required {
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired
            } else {
                WorthQueryManagedRunCleanupDisposition::CleanupComplete
            };
            let receipt = WorthQueryDirectReadmissionCleanupReceipt::from_completed(
                WorthQueryCompletedDirectReadmissionCleanup {
                    logical_run_identity: partial.logical_run_identity,
                    yielded_attempt_identity: partial.yielded_attempt_identity,
                    disposition,
                    checkpoint_release: partial.checkpoint_release,
                    restored_execution_release: partial.restored_execution_release,
                    bridge: bridge.release(),
                    relational: partial.relational,
                    attempt: partial.attempt,
                    run_counters: partial.run_counters,
                    provider_work: partial.provider_work,
                    yield_counters: partial.yield_counters,
                    readmission_evidence: partial.readmission_progress.evidence(),
                },
            );
            partial.resources.native_released = receipt.inspection().resources_released()
                && receipt
                    .inspection()
                    .provider_work()
                    .provider_retained_bytes()
                    == 0;
            if let Some(custody) = partial.custody {
                custody.record(
                    QueryManagedRunPosture::ReadmissionCleanup {
                        disposition,
                        resources: partial.resources,
                    },
                    receipt.inspection().yielded_attempt_identity(),
                );
            }
            if recovery_required {
                WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(receipt)
            } else {
                WorthQueryDirectReadmissionCleanupOutcome::Complete(receipt)
            }
        }
        BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(bridge) => {
            if let Some(custody) = &partial.custody {
                custody.record(
                    QueryManagedRunPosture::ReadmissionCleanup {
                        disposition: WorthQueryManagedRunCleanupDisposition::CleanupPending,
                        resources: partial.resources,
                    },
                    &partial.yielded_attempt_identity,
                );
            }
            partial
                .readmission_progress
                .observe_bridge(bridge.counters());
            WorthQueryDirectReadmissionCleanupOutcome::Pending(
                WorthQueryDirectReadmissionCleanupPending {
                    inspection: WorthQueryDirectReadmissionCleanupPendingInspection::capture(
                        &partial,
                    ),
                    receipt: partial,
                    bridge,
                },
            )
        }
    }
}

fn direct_cleanup_recovery_required(
    receipt: &WorthQueryDirectReadmissionPartialCleanupReceipt,
) -> bool {
    receipt.provider_work.provider_retained_bytes() != 0
        || receipt.checkpoint_release.disposition().recovery_required()
        || receipt
            .restored_execution_release
            .as_ref()
            .is_some_and(WorthQueryProviderExecutionReleaseEvidence::recovery_required)
}
