use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

use super::managed_graph_execution::WorthQueryManagedGraphExecution;
use super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::workflow::WorthQueryWorkflowRunAffinity;
use super::workflow_yield_eligibility::classify_workflow_yield_denial;
use super::workflow_yield_freeze::freeze_and_finalize_workflow_yield;
use super::workflow_yield_recovery::{
    terminalized_recovery, WorthQueryTerminalizedWorkflowYieldRecovery,
};
use super::{
    WorthQueryManagedRunCounters, WorthQueryPausedWorkflowGraphExecution,
    WorthQueryWorkflowYieldDenied, WorthQueryWorkflowYieldOutcome, WorthQueryYieldRecoveryKind,
    WorthQueryYieldRecoveryResourceEvidence, WorthQueryYieldTransitionCounters,
    WorthQueryYieldedWorkflowRun,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryFrozenWorkflowArtifactAuthority,
};

pub(super) fn yield_workflow_run(
    paused: WorthQueryPausedWorkflowGraphExecution,
) -> WorthQueryWorkflowYieldOutcome {
    let mut counters = WorthQueryYieldTransitionCounters::default();
    counters.classified_eligibility();
    if let Some((kind, detail)) = classify_workflow_yield_denial(&paused) {
        return WorthQueryWorkflowYieldOutcome::Denied(WorthQueryWorkflowYieldDenied {
            kind,
            detail: Arc::from(detail),
            paused,
            counters,
        });
    }
    let checkpoint_pending = match freeze_and_finalize_workflow_yield(paused, counters) {
        Ok(pending) => pending,
        Err(outcome) => return outcome,
    };
    let retained = match checkpoint_pending.suspend_checkpoint() {
        Ok(retained) => retained,
        Err(outcome) => return outcome,
    };
    retained.validate_and_mint()
}

pub(super) struct WorthQueryWorkflowYieldCheckpointPending {
    pub(super) affinity: WorthQueryWorkflowRunAffinity,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) run_counters: WorthQueryManagedRunCounters,
    pub(super) artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    pub(super) provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    pub(super) execution: WorthQueryManagedGraphExecution,
    pub(super) bridge: BridgeYieldedExecutionBasis,
    pub(super) yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldCheckpointPending {
    fn suspend_checkpoint(
        self,
    ) -> Result<WorthQueryWorkflowYieldRetained, WorthQueryWorkflowYieldOutcome> {
        let Self {
            mut affinity,
            relational_basis,
            run_counters,
            artifacts,
            provider_artifact_occurrences,
            execution,
            bridge,
            mut yield_counters,
        } = self;
        yield_counters.attempted_checkpoint_suspension();
        let retained = match execution.suspend() {
            Ok(suspension) => {
                yield_counters.observed_checkpoint_retained_bytes(1);
                affinity.record_provider_execution_release(&suspension.provider_execution_release);
                suspension.retained
            }
            Err(failure) => {
                yield_counters.observed_checkpoint_retained_bytes(
                    failure.checkpoint_retained_byte_probe_count(),
                );
                affinity.record_provider_execution_release(failure.provider_execution_release());
                affinity.abandon_provider_work();
                let kind = failure.kind();
                let detail = Arc::from(failure.detail());
                return Err(terminalized_recovery(
                    WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(kind),
                    detail,
                    yield_counters,
                    WorthQueryTerminalizedWorkflowYieldRecovery {
                        affinity: affinity.finish_yield(),
                        relational_basis,
                        artifacts,
                        bridge,
                        run_counters,
                    },
                    WorthQueryYieldRecoveryResourceEvidence::provider_checkpoint_suspension(
                        failure,
                    ),
                ));
            }
        };
        Ok(WorthQueryWorkflowYieldRetained {
            affinity,
            relational_basis,
            run_counters,
            artifacts,
            provider_artifact_occurrences,
            bridge,
            yield_counters,
            retained,
        })
    }

    pub(super) fn recovery(
        self,
        kind: WorthQueryYieldRecoveryKind,
        detail: Arc<str>,
    ) -> WorthQueryWorkflowYieldOutcome {
        terminalized_recovery(
            kind,
            detail,
            self.yield_counters,
            WorthQueryTerminalizedWorkflowYieldRecovery {
                affinity: self.affinity.finish_yield(),
                relational_basis: self.relational_basis,
                artifacts: self.artifacts,
                bridge: self.bridge,
                run_counters: self.run_counters,
            },
            WorthQueryYieldRecoveryResourceEvidence::default(),
        )
    }
}

struct WorthQueryWorkflowYieldRetained {
    affinity: WorthQueryWorkflowRunAffinity,
    relational_basis: WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    bridge: BridgeYieldedExecutionBasis,
    yield_counters: WorthQueryYieldTransitionCounters,
    retained: WorthQueryRetainedManagedGraphExecution,
}

pub(super) struct WorthQueryWorkflowYieldMint {
    _owner: (),
}

pub(super) struct WorthQueryWorkflowYieldMintedOwner {
    pub(super) affinity: super::workflow::WorthQueryWorkflowRunAffinity,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) bridge: worth_runtime_bridge::facade::BridgeYieldedExecutionBasis,
    pub(super) execution: super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
    pub(super) artifacts:
        crate::domain_computation::artifact_owner::WorthQueryFrozenWorkflowArtifactAuthority,
    pub(super) artifact_evidence:
        crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactRegistryEvidence,
    pub(super) run_counters: super::WorthQueryManagedRunCounters,
    pub(super) provider_artifact_occurrences: std::sync::Arc<
        crate::domain_computation::artifact_owner::WorthQueryArtifactOccurrenceLedger,
    >,
    pub(super) yield_counters: super::WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldMint {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

impl WorthQueryWorkflowYieldRetained {
    fn validate_and_mint(mut self) -> WorthQueryWorkflowYieldOutcome {
        self.yield_counters.observed_artifact_registry();
        let artifact_evidence = self.artifacts.evidence();
        let ceiling = self
            .affinity
            .operation_resources()
            .envelope()
            .yield_contract()
            .expect("eligibility established the installed yield contract")
            .retained_bytes_ceiling();
        let retained_total = self
            .retained
            .checkpoint_evidence()
            .retained_bytes()
            .saturating_add(u64::try_from(artifact_evidence.retained_bytes()).unwrap_or(u64::MAX));
        self.yield_counters.validated_retained_resources();
        if retained_total > ceiling {
            return self.recover_retained_bytes_exceeded();
        }
        self.affinity.interrupt_provider_step_call();
        self.affinity
            .settle_provider_artifacts(self.provider_artifact_occurrences.snapshot());
        self.yield_counters.minted_yielded_capability();
        WorthQueryWorkflowYieldOutcome::Yielded(
            WorthQueryYieldedWorkflowRun::owner_from_yield_transition(
                WorthQueryWorkflowYieldMintedOwner {
                    affinity: self.affinity,
                    relational_basis: self.relational_basis,
                    bridge: self.bridge,
                    execution: self.retained,
                    artifacts: self.artifacts,
                    artifact_evidence,
                    run_counters: self.run_counters,
                    provider_artifact_occurrences: self.provider_artifact_occurrences,
                    yield_counters: self.yield_counters,
                },
                WorthQueryWorkflowYieldMint::mint(),
            ),
        )
    }

    fn recover_retained_bytes_exceeded(self) -> WorthQueryWorkflowYieldOutcome {
        let Self {
            mut affinity,
            relational_basis,
            run_counters,
            artifacts,
            provider_artifact_occurrences: _,
            bridge,
            yield_counters,
            retained,
        } = self;
        let checkpoint_release = retained.release();
        let release_recovery_required = checkpoint_release.disposition().recovery_required();
        affinity.abandon_provider_work();
        terminalized_recovery(
            WorthQueryYieldRecoveryKind::RetainedBytesExceeded,
            Arc::from(if release_recovery_required {
                "over-ceiling checkpoint panicked during release"
            } else {
                "checkpoint and artifacts exceed the installed retained-byte ceiling"
            }),
            yield_counters,
            WorthQueryTerminalizedWorkflowYieldRecovery {
                affinity: affinity.finish_yield(),
                relational_basis,
                artifacts,
                bridge,
                run_counters,
            },
            WorthQueryYieldRecoveryResourceEvidence::retained_bytes_exceeded(checkpoint_release),
        )
    }
}
