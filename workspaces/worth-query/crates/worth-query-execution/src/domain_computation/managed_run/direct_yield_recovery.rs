use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

use super::{
    direct_yield_cleanup::{self, WorthQueryDirectYieldCleanupPermit},
    run_affinity::WorthQueryDirectRunTerminalAffinity,
    WorthQueryDirectYieldCleanupReceipt, WorthQueryDirectYieldOutcome,
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCounters,
    WorthQueryPausedDirectGraphExecution, WorthQueryYieldRecoveryKind,
    WorthQueryYieldRecoveryResourceEvidence, WorthQueryYieldTransitionCounters,
};

pub(super) enum WorthQueryDirectYieldRecoveryState {
    Running(WorthQueryPausedDirectGraphExecution),
    Terminalized {
        affinity: WorthQueryDirectRunTerminalAffinity,
        relational_basis: WorthQueryManagedRelationalObservation,
        bridge: BridgeYieldedExecutionBasis,
        run_counters: WorthQueryManagedRunCounters,
        provider_work: WorthQueryManagedProviderWorkEvidence,
        provider_cleanup: super::provider_work::WorthQueryManagedProviderCleanupAuthority,
    },
}

#[must_use = "direct yield recovery retains paused or terminal-cleanup authority"]
pub struct WorthQueryDirectYieldRecoveryRequired {
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    resource_evidence: WorthQueryYieldRecoveryResourceEvidence,
    state: WorthQueryDirectYieldRecoveryState,
}

impl WorthQueryDirectYieldRecoveryRequired {
    pub const fn kind(&self) -> WorthQueryYieldRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryYieldTransitionCounters {
        self.counters
    }

    pub fn resource_evidence(&self) -> &WorthQueryYieldRecoveryResourceEvidence {
        &self.resource_evidence
    }

    pub fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        match &self.state {
            WorthQueryDirectYieldRecoveryState::Running(paused) => &paused.active.running.counters,
            WorthQueryDirectYieldRecoveryState::Terminalized { run_counters, .. } => run_counters,
        }
    }

    pub fn running_attempt_recoverable(&self) -> bool {
        matches!(self.state, WorthQueryDirectYieldRecoveryState::Running(_))
    }

    #[must_use = "recovering a direct yielded run returns the exact paused owner or recovery owner"]
    pub fn into_paused(self) -> Result<WorthQueryPausedDirectGraphExecution, Self> {
        match self.state {
            WorthQueryDirectYieldRecoveryState::Running(paused) => Ok(paused),
            state => Err(Self {
                kind: self.kind,
                detail: self.detail,
                counters: self.counters,
                resource_evidence: self.resource_evidence,
                state,
            }),
        }
    }

    #[must_use = "terminalized direct yield cleanup returns a closed receipt or recovery owner"]
    pub fn cleanup_terminalized(self) -> Result<WorthQueryDirectYieldCleanupReceipt, Self> {
        direct_yield_cleanup::cleanup_terminalized(self)
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn owner_into_terminal_cleanup_parts(
        self,
        _owner: &WorthQueryDirectYieldCleanupPermit,
    ) -> Result<
        (
            Arc<str>,
            Arc<str>,
            WorthQueryDirectRunTerminalAffinity,
            WorthQueryManagedRelationalObservation,
            BridgeYieldedExecutionBasis,
            WorthQueryManagedRunCounters,
            WorthQueryManagedProviderWorkEvidence,
            super::provider_work::WorthQueryManagedProviderCleanupAuthority,
            WorthQueryYieldTransitionCounters,
            WorthQueryYieldRecoveryResourceEvidence,
        ),
        Self,
    > {
        let WorthQueryDirectYieldRecoveryState::Terminalized {
            affinity,
            relational_basis,
            bridge,
            run_counters,
            provider_work,
            provider_cleanup,
        } = self.state
        else {
            return Err(self);
        };
        let (logical_run_identity, attempt_identity) = affinity.terminal_descriptions();
        Ok((
            logical_run_identity,
            attempt_identity,
            affinity,
            relational_basis,
            bridge,
            run_counters,
            provider_work,
            provider_cleanup,
            self.counters,
            self.resource_evidence,
        ))
    }
}

pub(super) fn running_recovery(
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    paused: WorthQueryPausedDirectGraphExecution,
) -> WorthQueryDirectYieldOutcome {
    if let Some(custody) = paused.active.running.affinity.custody() {
        custody.record_recovery(crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery::Yield { cause: kind, state: crate::domain_computation::primary_graph::query_obligations::QueryManagedYieldRecoveryState::Running });
    }
    WorthQueryDirectYieldOutcome::RecoveryRequired(WorthQueryDirectYieldRecoveryRequired {
        kind,
        detail,
        counters,
        resource_evidence: WorthQueryYieldRecoveryResourceEvidence::default(),
        state: WorthQueryDirectYieldRecoveryState::Running(paused),
    })
}

pub(super) struct WorthQueryTerminalizedDirectYieldRecovery {
    pub(super) affinity: WorthQueryDirectRunTerminalAffinity,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) bridge: BridgeYieldedExecutionBasis,
    pub(super) run_counters: WorthQueryManagedRunCounters,
    pub(super) provider_work: WorthQueryManagedProviderWorkEvidence,
    pub(super) provider_cleanup: super::provider_work::WorthQueryManagedProviderCleanupAuthority,
}

pub(super) fn terminalized_recovery(
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    state: WorthQueryTerminalizedDirectYieldRecovery,
    resource_evidence: WorthQueryYieldRecoveryResourceEvidence,
) -> WorthQueryDirectYieldOutcome {
    if let Some(custody) = state.affinity.custody() {
        custody.record_recovery(crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery::Yield { cause: kind, state: crate::domain_computation::primary_graph::query_obligations::QueryManagedYieldRecoveryState::Terminalized });
    }
    WorthQueryDirectYieldOutcome::RecoveryRequired(WorthQueryDirectYieldRecoveryRequired {
        kind,
        detail,
        counters,
        resource_evidence,
        state: WorthQueryDirectYieldRecoveryState::Terminalized {
            affinity: state.affinity,
            relational_basis: state.relational_basis,
            bridge: state.bridge,
            run_counters: state.run_counters,
            provider_work: state.provider_work,
            provider_cleanup: state.provider_cleanup,
        },
    })
}
