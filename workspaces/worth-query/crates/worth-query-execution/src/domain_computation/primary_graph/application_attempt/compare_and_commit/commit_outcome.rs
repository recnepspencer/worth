//! Compare-and-commit outcome and denial taxonomy.

use super::WorthQueryApplicationCommitDeferred;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationNoEffectCause {
    OwnerDeniedBeforeEffect,
    CorrespondenceRebindRequired,
    ReferenceGenerationExhausted,
    CapacityExhausted,
    OwnerUnavailable,
    PreEffectFailure,
}

#[derive(Debug)]
pub struct WorthQueryApplicationNoEffect {
    terminal: worth_runtime_world::facade::NoEffectCompositePublication,
}

impl WorthQueryApplicationNoEffect {
    pub(in crate::domain_computation::primary_graph) fn from_world(
        terminal: worth_runtime_world::facade::NoEffectCompositePublication,
    ) -> Self {
        Self { terminal }
    }

    pub fn cause(&self) -> WorthQueryApplicationNoEffectCause {
        use worth_runtime_world::facade::NoEffectCause as Cause;
        match self.terminal.cause() {
            Cause::OwnerDeniedBeforeEffect => {
                WorthQueryApplicationNoEffectCause::OwnerDeniedBeforeEffect
            }
            Cause::CorrespondenceRebindRequired => {
                WorthQueryApplicationNoEffectCause::CorrespondenceRebindRequired
            }
            Cause::ReferenceGenerationExhausted => {
                WorthQueryApplicationNoEffectCause::ReferenceGenerationExhausted
            }
            Cause::CapacityExhausted => WorthQueryApplicationNoEffectCause::CapacityExhausted,
            Cause::OwnerUnavailable => WorthQueryApplicationNoEffectCause::OwnerUnavailable,
            Cause::PreEffectFailure => WorthQueryApplicationNoEffectCause::PreEffectFailure,
            Cause::StaleExpectedProductHead
            | Cause::CancelledBeforeEffect
            | Cause::DeadlineBeforeEffect => {
                unreachable!("stale, cancellation, and deadline have dedicated Query outcomes")
            }
        }
    }
}

mod settlement_deferred;
pub use settlement_deferred::{
    WorthQueryApplicationSettlementDeferred, WorthQueryApplicationSettlementNextAction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationStaleAttempt {
    stale_fact_count: usize,
}

impl WorthQueryApplicationStaleAttempt {
    pub const fn stale_fact_count(self) -> usize {
        self.stale_fact_count
    }

    pub(in super::super) const fn new(stale_fact_count: usize) -> Self {
        Self { stale_fact_count }
    }
}

mod denial;
pub(in crate::domain_computation::primary_graph) use denial::WorthQueryProgramActivationUnresolved;
pub use denial::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

#[derive(Debug)]
pub enum WorthQueryApplicationCommitOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffect),
    Committed(super::WorthQueryApplicationCommitReceipt),
    AlreadyCommitted(super::WorthQueryApplicationCommitReceipt),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate(WorthQueryApplicationUnresolvedCommitEvidence),
}

impl WorthQueryApplicationCommitOutcome {
    /// Requires a committed application result while returning every other
    /// typed terminal with its recovery custody intact.
    pub fn require_committed(self) -> Result<super::WorthQueryApplicationCommitReceipt, Self> {
        match self {
            Self::Committed(receipt) | Self::AlreadyCommitted(receipt) => Ok(receipt),
            other => Err(other),
        }
    }

    /// Splits a landed commit from every terminal that did not land. The flag
    /// is true when the landing replays an earlier commit of the same
    /// idempotency.
    pub fn landed(
        self,
    ) -> Result<(super::WorthQueryApplicationCommitReceipt, bool), WorthQueryApplicationUncommitted>
    {
        use WorthQueryApplicationUncommitted as Uncommitted;
        Err(match self {
            Self::Committed(receipt) => return Ok((receipt, false)),
            Self::AlreadyCommitted(receipt) => return Ok((receipt, true)),
            Self::ProductStale(stale) => Uncommitted::ProductStale(stale),
            Self::ProductUnpublished(unpublished) => Uncommitted::ProductUnpublished(unpublished),
            Self::NoEffect(no_effect) => Uncommitted::NoEffect(no_effect),
            Self::Stale(stale) => Uncommitted::Stale(stale),
            Self::Cancelled => Uncommitted::Cancelled,
            Self::TimedOut => Uncommitted::TimedOut,
            Self::Denied(denial) => Uncommitted::Denied(denial),
            Self::Aborted => Uncommitted::Aborted,
            Self::Deferred(deferred) => Uncommitted::Deferred(deferred),
            Self::SettlementDeferred(deferred) => Uncommitted::SettlementDeferred(deferred),
            Self::Indeterminate(evidence) => Uncommitted::Indeterminate(evidence),
        })
    }
}

/// Every commit terminal except a landed commit. An outcome that reports its
/// own landed shape carries this in place of the whole commit outcome, so each
/// variant a caller matches can occur. `Indeterminate` means the landing is
/// unresolved, not that it failed.
#[derive(Debug)]
pub enum WorthQueryApplicationUncommitted {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffect),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate(WorthQueryApplicationUnresolvedCommitEvidence),
}

impl From<WorthQueryApplicationUncommitted> for WorthQueryApplicationCommitOutcome {
    fn from(uncommitted: WorthQueryApplicationUncommitted) -> Self {
        use WorthQueryApplicationUncommitted as Uncommitted;
        match uncommitted {
            Uncommitted::ProductStale(stale) => Self::ProductStale(stale),
            Uncommitted::ProductUnpublished(unpublished) => Self::ProductUnpublished(unpublished),
            Uncommitted::NoEffect(no_effect) => Self::NoEffect(no_effect),
            Uncommitted::Stale(stale) => Self::Stale(stale),
            Uncommitted::Cancelled => Self::Cancelled,
            Uncommitted::TimedOut => Self::TimedOut,
            Uncommitted::Denied(denial) => Self::Denied(denial),
            Uncommitted::Aborted => Self::Aborted,
            Uncommitted::Deferred(deferred) => Self::Deferred(deferred),
            Uncommitted::SettlementDeferred(deferred) => Self::SettlementDeferred(deferred),
            Uncommitted::Indeterminate(evidence) => Self::Indeterminate(evidence),
        }
    }
}

/// Correlation evidence retained when commit outcome is unresolved (R8.26 / C3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationUnresolvedCommitEvidence {
    recovery: WorthQueryApplicationCommitRecoveryKind,
    denial_kind: crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind,
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
}

/// Distinguishes commit-path vs abort-path recovery requirement (R8.26).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitRecoveryKind {
    CommitRecoveryRequired,
    AbortRecoveryRequired,
}

impl WorthQueryApplicationUnresolvedCommitEvidence {
    pub(in crate::domain_computation::primary_graph) fn from_provider_session_failure(
        recovery: WorthQueryApplicationCommitRecoveryKind,
        failure: &crate::domain_computation::provider_session::WorthQueryProviderSessionFailure,
    ) -> Self {
        Self {
            recovery,
            denial_kind: failure.kind(),
            stage: failure.stage(),
            detail: failure.detail().to_owned(),
        }
    }

    pub const fn recovery(&self) -> WorthQueryApplicationCommitRecoveryKind {
        self.recovery
    }

    pub const fn denial_kind(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind {
        self.denial_kind
    }

    pub const fn stage(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
