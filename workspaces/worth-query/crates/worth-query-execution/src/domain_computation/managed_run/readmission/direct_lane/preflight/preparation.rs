use std::sync::Arc;

use crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionOutcome, BridgeExecutionBasisReadmissionPending,
    BridgeYieldedExecutionBasis, BridgeYieldedExecutionBasisPreflight, RuntimeBridge,
};

use super::super::super::direct_outcome::{
    WorthQueryDirectReadmissionDenialKind, WorthQueryDirectReadmissionDenied,
    WorthQueryDirectReadmissionOutcome,
};
use super::super::super::evidence::WorthQueryReadmissionProgress;
pub(super) mod progression;
mod validation;

use crate::domain_computation::managed_run::{
    retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
    run_affinity::{WorthQueryDirectRunAffinity, WorthQueryDirectRunReadmissionPending},
    step_contract_admission::WorthQueryAdmittedManagedStepContract,
    WorthQueryManagedRunCounters, WorthQueryYieldTransitionCounters, WorthQueryYieldedDirectRun,
};
use crate::domain_computation::WorthQueryExecutionRuntime;
use progression::bridge_cleanup_recovery_required;
use validation::validate_direct_resume_preflight;

struct WorthQueryDirectYieldedState {
    affinity: WorthQueryDirectRunAffinity,
    retained: WorthQueryDirectRetainedState,
}

struct WorthQueryDirectRetainedState {
    relational_basis: WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
    inspection: crate::domain_computation::WorthQueryYieldedDirectRunInspection,
}

struct WorthQueryDirectYieldedParts {
    state: WorthQueryDirectYieldedState,
    bridge: BridgeYieldedExecutionBasis,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryDirectYieldRestoredOwner {
    pub(in crate::domain_computation::managed_run) affinity: WorthQueryDirectRunAffinity,
    pub(in crate::domain_computation::managed_run) relational_basis:
        WorthQueryManagedRelationalObservation,
    pub(in crate::domain_computation::managed_run) bridge: BridgeYieldedExecutionBasis,
    pub(in crate::domain_computation::managed_run) execution:
        WorthQueryRetainedManagedGraphExecution,
    pub(in crate::domain_computation::managed_run) run_counters: WorthQueryManagedRunCounters,
    pub(in crate::domain_computation::managed_run) yield_counters:
        WorthQueryYieldTransitionCounters,
    pub(in crate::domain_computation::managed_run) inspection:
        crate::domain_computation::WorthQueryYieldedDirectRunInspection,
}

struct WorthQueryDirectProvisionalResourceAttempt {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryRetainedManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeYieldedExecutionBasisPreflight,
    contract: WorthQueryAdmittedManagedStepContract,
}

struct WorthQueryDirectBridgeReadmissionAttempt {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryRetainedManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
    contract: WorthQueryAdmittedManagedStepContract,
}

pub(super) struct WorthQueryDirectBridgeReadmissionPending {
    state: WorthQueryDirectRetainedState,
    execution: WorthQueryRetainedManagedGraphExecution,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
    contract: WorthQueryAdmittedManagedStepContract,
}

pub(in crate::domain_computation::managed_run::readmission) fn prepare_direct_provider_restore(
    yielded: WorthQueryYieldedDirectRun,
    query_runtime: &WorthQueryExecutionRuntime,
    bridge_runtime: &RuntimeBridge,
) -> Result<
    (
        WorthQueryDirectBridgeReadmissionPending,
        WorthQueryReadmissionProgress,
    ),
    WorthQueryDirectReadmissionOutcome,
> {
    let mut progress = WorthQueryReadmissionProgress::default();
    progress.checked_preflight();
    let preflight = match validate_direct_resume_preflight(yielded, query_runtime, bridge_runtime) {
        Ok(preflight) => preflight,
        Err(denial) => {
            let (kind, detail, yielded, bridge_counters) = denial.into_parts();
            if let Some(bridge_counters) = bridge_counters {
                progress.observe_bridge(bridge_counters);
            }
            return Err(denied(kind, detail, yielded, progress));
        }
    };
    let provisional = preflight.begin_resource_attempt(&mut progress);
    let pending = match begin_bridge_readmission(provisional, bridge_runtime, progress) {
        Ok((pending, next_progress)) => {
            progress = next_progress;
            pending
        }
        Err(outcome) => return Err(outcome),
    };
    Ok((pending, progress))
}

fn begin_bridge_readmission(
    provisional: WorthQueryDirectProvisionalResourceAttempt,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
) -> Result<
    (
        WorthQueryDirectBridgeReadmissionPending,
        WorthQueryReadmissionProgress,
    ),
    WorthQueryDirectReadmissionOutcome,
> {
    let WorthQueryDirectProvisionalResourceAttempt {
        state,
        execution,
        resource,
        bridge,
        contract,
    } = provisional;
    let intent = resource.bridge_readmission_intent(
        &super::super::WorthQueryDirectReadmissionTransitionPermit::mint(),
    );
    let attempt = WorthQueryDirectBridgeReadmissionAttempt {
        state,
        execution,
        resource,
        contract,
    };
    progress.attempted_bridge_readmission();
    match bridge_runtime.readmit_yielded_execution_basis(bridge, intent) {
        BridgeExecutionBasisReadmissionOutcome::Pending(bridge) => {
            progress.observe_bridge(bridge.counters());
            Ok((
                WorthQueryDirectBridgeReadmissionPending {
                    state: attempt.state,
                    execution: attempt.execution,
                    resource: attempt.resource,
                    bridge,
                    contract: attempt.contract,
                },
                progress,
            ))
        }
        BridgeExecutionBasisReadmissionOutcome::Denied(denial) => {
            let detail = Arc::from(denial.detail());
            let (bridge, bridge_counters) = denial.into_returned_yielded().into_parts();
            progress.observe_bridge(bridge_counters);
            Err(denied(
                WorthQueryDirectReadmissionDenialKind::BridgeReadmissionDenied,
                detail,
                WorthQueryDirectYieldedParts {
                    state: WorthQueryDirectYieldedState {
                        affinity: attempt.resource.abort(
                            super::super::WorthQueryDirectReadmissionTransitionPermit::mint(),
                        ),
                        retained: attempt.state,
                    },
                    bridge,
                    execution: attempt.execution,
                }
                .into_yielded(),
                progress,
            ))
        }
        BridgeExecutionBasisReadmissionOutcome::RecoveryRequired(recovery) => {
            let detail = Arc::from(recovery.detail());
            progress.observe_bridge(recovery.counters());
            Err(WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                bridge_cleanup_recovery_required(detail, progress, attempt, recovery),
            ))
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

impl WorthQueryDirectYieldedParts {
    fn from_yielded(yielded: WorthQueryYieldedDirectRun) -> Self {
        let (
            affinity,
            relational_basis,
            bridge,
            execution,
            run_counters,
            yield_counters,
            inspection,
        ) = yielded.owner_into_readmission_parts(
            &super::super::WorthQueryDirectReadmissionTransitionPermit::mint(),
        );
        Self {
            state: WorthQueryDirectYieldedState {
                affinity,
                retained: WorthQueryDirectRetainedState {
                    relational_basis,
                    run_counters,
                    yield_counters,
                    inspection,
                },
            },
            bridge,
            execution,
        }
    }

    fn into_yielded(self) -> WorthQueryYieldedDirectRun {
        WorthQueryYieldedDirectRun::owner_restore_from_readmission(
            WorthQueryDirectYieldRestoredOwner {
                affinity: self.state.affinity,
                relational_basis: self.state.retained.relational_basis,
                bridge: self.bridge,
                execution: self.execution,
                run_counters: self.state.retained.run_counters,
                yield_counters: self.state.retained.yield_counters,
                inspection: self.state.retained.inspection,
            },
            &super::super::WorthQueryDirectReadmissionTransitionPermit::mint(),
        )
    }
}
