//! Compare-and-commit outcome and denial taxonomy.

use super::WorthQueryApplicationCommitDeferred;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationNoEffectCause {
    ExecutionRequest(worth_execution::WorkCeilingDenial),
    OwnerDeniedBeforeEffect,
    CorrespondenceRebindRequired,
    ReferenceGenerationExhausted,
    CapacityExhausted,
    OwnerUnavailable,
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    PreEffectFailure,
}

pub struct WorthQueryApplicationNoEffect {
    terminal: worth_runtime_world::facade::NoEffectCompositePublication,
    cause: WorthQueryApplicationNoEffectCause,
}

impl std::fmt::Debug for WorthQueryApplicationNoEffect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Terminal custody owns persistent graph roots. Diagnostic rendering
        // describes the cause without recursively traversing that storage.
        formatter
            .debug_struct("WorthQueryApplicationNoEffect")
            .field("cause", &self.terminal.cause())
            .finish_non_exhaustive()
    }
}

impl WorthQueryApplicationNoEffect {
    pub(in crate::domain_computation::primary_graph) fn from_world(
        terminal: worth_runtime_world::facade::NoEffectCompositePublication,
        cause: WorthQueryApplicationNoEffectCause,
    ) -> Self {
        Self { terminal, cause }
    }

    pub fn cause(&self) -> WorthQueryApplicationNoEffectCause {
        self.cause
    }
}

/// Evidence that sealed decision facts changed at the admitted commit basis.
/// Nothing was committed.
///
/// Re-read from a fresh basis and retry with a fresh source. The runtime never
/// reruns the handler for you.
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

/// Every way an application compare-and-commit can end.
///
/// `Committed` and `AlreadyCommitted` landed and carry the receipt. Every other
/// variant did not land cleanly; see [`landed`](Self::landed) to split the two
/// sides. `Indeterminate` may have moved the branch: it is unresolved, not
/// failed, and does not call for a blind retry.
#[derive(Debug)]
pub enum WorthQueryApplicationCommitOutcome {
    /// Business evidence and actual refused managed-cleanup custody remain separate.
    CleanupRequired(Box<super::WorthQueryApplicationCleanupRequired>),
    /// The product branch moved after the attempt's basis. Nothing was committed;
    /// re-read and retry.
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    /// Some component owners moved, but the product head did not. Recover the
    /// product publication.
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    /// Nothing was written. The cause says why.
    NoEffect(WorthQueryApplicationNoEffect),
    /// This attempt committed; the receipt says what landed.
    Committed(super::WorthQueryApplicationCommitReceipt),
    /// The idempotency key had already committed the same intent; the receipt is
    /// the earlier commit's, and the change was not made again.
    AlreadyCommitted(super::WorthQueryApplicationCommitReceipt),
    /// The candidate's basis was stale at compare. Nothing was committed; re-read and
    /// retry.
    Stale(WorthQueryApplicationStaleAttempt),
    /// The attempt was canceled before it landed.
    Cancelled,
    /// The attempt reached its deadline before it landed.
    TimedOut,
    /// The commit was refused before publication, with a typed kind and stage.
    Denied(WorthQueryApplicationCommitDenial),
    /// The commit's answer was lost, and a re-read through the idempotency key found
    /// that it never landed.
    Aborted,
    /// A capacity or lifetime limit stopped the attempt before it landed. Retry
    /// later.
    Deferred(WorthQueryApplicationCommitDeferred),
    /// Whether the commit landed is unresolved. Recover in the direction the
    /// evidence names.
    Indeterminate(WorthQueryApplicationUnresolvedCommitEvidence),
}

impl WorthQueryApplicationCommitOutcome {
    /// Requires a committed application result while returning every other
    /// typed terminal with its recovery custody intact.
    pub fn require_committed(self) -> Result<super::WorthQueryApplicationCommitReceipt, Self> {
        match self {
            Self::Committed(receipt) | Self::AlreadyCommitted(receipt) => Ok(receipt),
            other @ (Self::CleanupRequired(_)
            | Self::ProductStale(_)
            | Self::ProductUnpublished(_)
            | Self::NoEffect(_)
            | Self::Stale(_)
            | Self::Cancelled
            | Self::TimedOut
            | Self::Denied(_)
            | Self::Aborted
            | Self::Deferred(_)
            | Self::Indeterminate(_)) => Err(other),
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
            Self::CleanupRequired(cleanup) => Uncommitted::CleanupRequired(cleanup),
            Self::ProductStale(stale) => Uncommitted::ProductStale(stale),
            Self::ProductUnpublished(unpublished) => Uncommitted::ProductUnpublished(unpublished),
            Self::NoEffect(no_effect) => Uncommitted::NoEffect(no_effect),
            Self::Stale(stale) => Uncommitted::Stale(stale),
            Self::Cancelled => Uncommitted::Cancelled,
            Self::TimedOut => Uncommitted::TimedOut,
            Self::Denied(denial) => Uncommitted::Denied(denial),
            Self::Aborted => Uncommitted::Aborted,
            Self::Deferred(deferred) => Uncommitted::Deferred(deferred),
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
    CleanupRequired(Box<super::WorthQueryApplicationCleanupRequired>),
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffect),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    Indeterminate(WorthQueryApplicationUnresolvedCommitEvidence),
}

impl From<WorthQueryApplicationUncommitted> for WorthQueryApplicationCommitOutcome {
    fn from(uncommitted: WorthQueryApplicationUncommitted) -> Self {
        use WorthQueryApplicationUncommitted as Uncommitted;
        match uncommitted {
            Uncommitted::CleanupRequired(cleanup) => Self::CleanupRequired(cleanup),
            Uncommitted::ProductStale(stale) => Self::ProductStale(stale),
            Uncommitted::ProductUnpublished(unpublished) => Self::ProductUnpublished(unpublished),
            Uncommitted::NoEffect(no_effect) => Self::NoEffect(no_effect),
            Uncommitted::Stale(stale) => Self::Stale(stale),
            Uncommitted::Cancelled => Self::Cancelled,
            Uncommitted::TimedOut => Self::TimedOut,
            Uncommitted::Denied(denial) => Self::Denied(denial),
            Uncommitted::Aborted => Self::Aborted,
            Uncommitted::Deferred(deferred) => Self::Deferred(deferred),
            Uncommitted::Indeterminate(evidence) => Self::Indeterminate(evidence),
        }
    }
}

/// Correlation evidence retained when commit outcome is unresolved (R8.26 / C3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationUnresolvedCommitEvidence {
    recovery: WorthQueryApplicationCommitRecoveryKind,
    denial_kind: Result<
        crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
    cleanup_failure: Option<WorthQueryApplicationManagedCleanupFailure>,
}

/// Additional managed cleanup evidence retained alongside the unresolved commit cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationManagedCleanupFailure {
    RunFinishRejected,
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
            denial_kind: failure.checked_read_stop().unwrap_or(Ok(failure.kind())),
            stage: failure.stage(),
            detail: failure.detail().to_owned(),
            cleanup_failure: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_read_stop(
        mut self,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ) -> Self {
        self.denial_kind = cause;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn with_cleanup_failure(mut self) -> Self {
        self.cleanup_failure = Some(WorthQueryApplicationManagedCleanupFailure::RunFinishRejected);
        self
    }
    pub const fn cleanup_failure(&self) -> Option<WorthQueryApplicationManagedCleanupFailure> {
        self.cleanup_failure
    }

    pub const fn recovery(&self) -> WorthQueryApplicationCommitRecoveryKind {
        self.recovery
    }

    pub const fn denial_kind(
        &self,
    ) -> Result<
        crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    > {
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
