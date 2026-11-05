use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeExecutionBasisSignalTerminal, BridgeYieldedExecutionBasis,
};

pub(super) struct WorthQueryDirectYieldMint {
    _owner: (),
}

impl WorthQueryDirectYieldMint {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

pub(super) struct WorthQueryDirectYieldMintedOwner {
    pub(super) affinity: super::run_affinity::WorthQueryDirectRunAffinity,
    pub(super) relational_basis: super::WorthQueryManagedRelationalObservation,
    pub(super) bridge: worth_runtime_bridge::facade::BridgeYieldedExecutionBasis,
    pub(super) execution: super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
    pub(super) run_counters:
        crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters,
    pub(super) yield_counters: super::WorthQueryYieldTransitionCounters,
}

use super::direct_yield_eligibility::classify_direct_yield_denial;
use super::direct_yield_recovery::{
    running_recovery, terminalized_recovery, WorthQueryTerminalizedDirectYieldRecovery,
};
use super::managed_graph_execution::WorthQueryManagedGraphExecution;
use super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::run_affinity::WorthQueryDirectRunAffinity;
use super::yield_eligibility::WorthQueryManagedYieldSafePoint;
use {
    super::WorthQueryDirectYieldDenied, super::WorthQueryDirectYieldOutcome,
    super::WorthQueryPausedDirectGraphExecution, super::WorthQueryRunningDirectRun,
    super::WorthQueryYieldRecoveryKind, super::WorthQueryYieldRecoveryResourceEvidence,
    super::WorthQueryYieldTransitionCounters, super::WorthQueryYieldedDirectRun,
    crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters,
};

pub(super) fn yield_direct_run(
    paused: WorthQueryPausedDirectGraphExecution,
) -> WorthQueryDirectYieldOutcome {
    let mut counters = WorthQueryYieldTransitionCounters::default();
    counters.classified_eligibility();
    if let Some((kind, detail)) = classify_direct_yield_denial(&paused) {
        return WorthQueryDirectYieldOutcome::Denied(WorthQueryDirectYieldDenied {
            kind,
            detail: Arc::from(detail),
            paused,
            counters,
        });
    }
    let bridge_pending = WorthQueryDirectYieldBridgePending::from_paused(paused, counters);
    let checkpoint_pending = match bridge_pending.finalize_bridge() {
        Ok(pending) => pending,
        Err(outcome) => return outcome,
    };
    let retained = match checkpoint_pending.suspend_checkpoint() {
        Ok(retained) => retained,
        Err(outcome) => return outcome,
    };
    retained.validate_and_mint()
}

struct WorthQueryDirectYieldBridgePending {
    affinity: WorthQueryDirectRunAffinity,
    bridge_basis: worth_runtime_bridge::facade::BridgeBoundExecutionBasis,
    relational_basis: super::WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    execution: WorthQueryManagedGraphExecution,
    safe_point: WorthQueryManagedYieldSafePoint,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryDirectYieldBridgePending {
    fn from_paused(
        paused: WorthQueryPausedDirectGraphExecution,
        yield_counters: WorthQueryYieldTransitionCounters,
    ) -> Self {
        let WorthQueryPausedDirectGraphExecution { active, safe_point } = paused;
        let super::WorthQueryActiveDirectGraphExecution { running, execution } = active;
        let WorthQueryRunningDirectRun {
            affinity,
            bridge_basis,
            relational_basis,
            counters,
        } = running;
        Self {
            affinity,
            bridge_basis,
            relational_basis,
            run_counters: counters,
            execution,
            safe_point,
            yield_counters,
        }
    }

    fn finalize_bridge(
        mut self,
    ) -> Result<WorthQueryDirectYieldCheckpointPending, WorthQueryDirectYieldOutcome> {
        self.yield_counters.attempted_bridge_finalization();
        let bridge = match self.bridge_basis.yield_execution_basis() {
            Ok(receipt) => receipt,
            Err(failure) => {
                let kind = failure.kind();
                let detail = Arc::from(failure.detail());
                let running = WorthQueryRunningDirectRun {
                    affinity: self.affinity,
                    bridge_basis: failure.into_basis(),
                    relational_basis: self.relational_basis,
                    counters: self.run_counters,
                };
                return Err(running_recovery(
                    WorthQueryYieldRecoveryKind::BridgeTerminalization(kind),
                    detail,
                    self.yield_counters,
                    WorthQueryPausedDirectGraphExecution {
                        active: super::WorthQueryActiveDirectGraphExecution {
                            running,
                            execution: self.execution,
                        },
                        safe_point: self.safe_point,
                    },
                ));
            }
        };
        let mut pending = WorthQueryDirectYieldCheckpointPending {
            affinity: self.affinity,
            relational_basis: self.relational_basis,
            run_counters: self.run_counters,
            execution: self.execution,
            bridge,
            yield_counters: self.yield_counters,
        };
        if !pending.bridge.receipt().signal_transition_performed()
            || pending.bridge.receipt().signal_terminal()
                != BridgeExecutionBasisSignalTerminal::Cancelled
        {
            pending.affinity.provider_work_mut().interrupt_step_call();
            let terminal = pending.bridge.receipt().signal_terminal();
            return Err(pending.recovery(
                WorthQueryYieldRecoveryKind::SignalAttemptAlreadyTerminal(terminal),
                Arc::from(
                    "Signal attempt terminalized before Query could perform the yield transition",
                ),
            ));
        }
        Ok(pending)
    }
}

struct WorthQueryDirectYieldCheckpointPending {
    affinity: WorthQueryDirectRunAffinity,
    relational_basis: super::WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    execution: WorthQueryManagedGraphExecution,
    bridge: BridgeYieldedExecutionBasis,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryDirectYieldCheckpointPending {
    fn suspend_checkpoint(
        self,
    ) -> Result<WorthQueryDirectYieldRetained, WorthQueryDirectYieldOutcome> {
        let Self {
            mut affinity,
            relational_basis,
            run_counters,
            execution,
            bridge,
            mut yield_counters,
        } = self;
        yield_counters.attempted_checkpoint_suspension();
        let retained = match execution.suspend() {
            Ok(suspension) => {
                yield_counters.observed_checkpoint_retained_bytes(1);
                affinity
                    .provider_work_mut()
                    .record_provider_execution_release(&suspension.provider_execution_release);
                suspension.retained
            }
            Err(failure) => {
                yield_counters.observed_checkpoint_retained_bytes(
                    failure.checkpoint_retained_byte_probe_count(),
                );
                affinity
                    .provider_work_mut()
                    .record_provider_execution_release(failure.provider_execution_release());
                affinity.provider_work_mut().abandon();
                let kind = failure.kind();
                let detail = Arc::from(failure.detail());
                let (affinity, provider_work, _) = affinity.into_terminal_parts();
                return Err(terminalized_recovery(
                    WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(kind),
                    detail,
                    yield_counters,
                    WorthQueryTerminalizedDirectYieldRecovery {
                        affinity,
                        relational_basis,
                        bridge,
                        run_counters,
                        provider_work,
                    },
                    WorthQueryYieldRecoveryResourceEvidence::provider_checkpoint_suspension(
                        failure,
                    ),
                ));
            }
        };
        Ok(WorthQueryDirectYieldRetained {
            affinity,
            relational_basis,
            run_counters,
            bridge,
            yield_counters,
            retained,
        })
    }

    fn recovery(
        self,
        kind: WorthQueryYieldRecoveryKind,
        detail: Arc<str>,
    ) -> WorthQueryDirectYieldOutcome {
        let (affinity, provider_work, _) = self.affinity.into_terminal_parts();
        terminalized_recovery(
            kind,
            detail,
            self.yield_counters,
            WorthQueryTerminalizedDirectYieldRecovery {
                affinity,
                relational_basis: self.relational_basis,
                bridge: self.bridge,
                run_counters: self.run_counters,
                provider_work,
            },
            WorthQueryYieldRecoveryResourceEvidence::default(),
        )
    }
}

struct WorthQueryDirectYieldRetained {
    affinity: WorthQueryDirectRunAffinity,
    relational_basis: super::WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    bridge: BridgeYieldedExecutionBasis,
    yield_counters: WorthQueryYieldTransitionCounters,
    retained: WorthQueryRetainedManagedGraphExecution,
}

impl WorthQueryDirectYieldRetained {
    fn validate_and_mint(mut self) -> WorthQueryDirectYieldOutcome {
        let ceiling = self
            .affinity
            .yield_retained_bytes_ceiling()
            .expect("eligibility established the installed yield contract");
        self.yield_counters.validated_retained_resources();
        if self.retained.checkpoint_evidence().retained_bytes() > ceiling {
            let checkpoint_release = self.retained.release();
            let release_recovery_required = checkpoint_release.disposition().recovery_required();
            self.affinity.provider_work_mut().abandon();
            let (affinity, provider_work, _) = self.affinity.into_terminal_parts();
            return terminalized_recovery(
                WorthQueryYieldRecoveryKind::RetainedBytesExceeded,
                Arc::from(if release_recovery_required {
                    "provider checkpoint exceeds the retained-byte ceiling and panicked during release"
                } else {
                    "provider checkpoint exceeds the installed retained-byte ceiling"
                }),
                self.yield_counters,
                WorthQueryTerminalizedDirectYieldRecovery {
                    affinity,
                    relational_basis: self.relational_basis,
                    bridge: self.bridge,
                    run_counters: self.run_counters,
                    provider_work,
                },
                WorthQueryYieldRecoveryResourceEvidence::retained_bytes_exceeded(
                    checkpoint_release,
                ),
            );
        }
        self.affinity.provider_work_mut().interrupt_step_call();
        self.yield_counters.minted_yielded_capability();
        WorthQueryDirectYieldOutcome::Yielded(
            WorthQueryYieldedDirectRun::owner_from_yield_transition(
                WorthQueryDirectYieldMintedOwner {
                    affinity: self.affinity,
                    relational_basis: self.relational_basis,
                    bridge: self.bridge,
                    execution: self.retained,
                    run_counters: self.run_counters,
                    yield_counters: self.yield_counters,
                },
                WorthQueryDirectYieldMint::mint(),
            ),
        )
    }
}
