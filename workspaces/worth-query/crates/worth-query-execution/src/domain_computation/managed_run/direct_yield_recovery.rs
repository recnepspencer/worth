use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

use {super::direct_yield_cleanup, super::direct_yield_cleanup::WorthQueryDirectYieldCleanupPermit, super::run_affinity::WorthQueryDirectRunTerminalAffinity, super::WorthQueryDirectYieldCleanupReceipt, super::WorthQueryDirectYieldOutcome, crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence, crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters, super::WorthQueryPausedDirectGraphExecution, super::WorthQueryYieldRecoveryKind, super::WorthQueryYieldRecoveryResourceEvidence, super::WorthQueryYieldTransitionCounters};

pub(super) enum WorthQueryDirectYieldRecoveryState {
    Running(WorthQueryPausedDirectGraphExecution),
    Terminalized {
        affinity: WorthQueryDirectRunTerminalAffinity,
        relational_basis: WorthQueryManagedRelationalObservation,
        bridge: BridgeYieldedExecutionBasis,
        run_counters: WorthQueryManagedRunCounters,
        provider_work: WorthQueryManagedProviderWorkEvidence,
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
}

pub(super) fn terminalized_recovery(
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    state: WorthQueryTerminalizedDirectYieldRecovery,
    resource_evidence: WorthQueryYieldRecoveryResourceEvidence,
) -> WorthQueryDirectYieldOutcome {
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
        },
    })
}
