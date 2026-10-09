use std::sync::Arc;
mod native_denial;
use native_denial::NativeDenial;
use worth_execution::ExecutionAllocationDenial;

/// Why an installed custom invariant refused a candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryCustomInvariantDenial {
    /// The invariant evaluated and found a violation.
    Violation {
        /// The invariant that found the violation.
        identity: worth_relational::facade::transactions::CustomInvariantSemanticIdentity,
    },
    /// The invariant could not be evaluated; the phase and failure kind say where
    /// and why.
    Failure {
        /// The invariant that failed.
        identity: worth_relational::facade::transactions::CustomInvariantFailureIdentity,
        /// Where evaluation failed.
        phase: worth_relational::facade::transactions::CustomInvariantFailurePhase,
        /// Why evaluation failed.
        failure: worth_relational::facade::transactions::ResultCustomInvariantFailureKind,
    },
}

/// Why invariant execution for a provisional attempt stopped.
///
/// Variants with fields carry the bound that was hit and, where known, the
/// amount the attempt needed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantExecutionDenialKind {
    /// Validation was refused by the execution owner before effects.
    ExecutionDenied(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// Validation was stopped by cancellation or its deadline.
    ExecutionControlStopped(crate::domain_computation::WorthQueryProviderSessionControlStopKind),
    /// Physical backing refusal; the original owner cause is retained separately.
    AllocationDenied,
    /// No installed invariant requirement exists for the requested slot.
    InvariantNotInstalled,
    /// The requirement's executor role is not the role of the provider running
    /// this attempt.
    ExecutorRoleMismatch,
    /// The state-load plan names a family the invariant did not declare.
    UndeclaredStateLoadFamily,
    /// The state-load plan asks for more facts than the invariant's declared
    /// maximum.
    StateLoadBudgetExceeded,
    /// Loading and executing the invariant used more work units than its
    /// budget allows.
    ExecutionBudgetExceeded,
    /// The provider has no port for this kind of invariant execution.
    ProviderUnsupported,
    /// The provider, or Relational behind it, refused the load, the touches, or
    /// the verdict. The failure's detail says which.
    ProviderRejected,
    /// The admitted request interrupted actual Relational validation.
    RequestInterrupted(worth_relational::facade::mvcc::RelationalOperationInterruption),
    /// Relational deferred publication until its required derived companion can proceed.
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    /// A custom invariant refused the candidate; see
    /// [`WorthQueryInvariantExecutionFailure::custom_invariant_denial`].
    CustomInvariantDenied,
    /// The product basis the attempt was prepared against is no longer current.
    ProductBasisStale,
    /// Relational has no room to retain another observation.
    RetentionCapacityExhausted,
    /// Relational has no retention identities left.
    RetentionIdentityExhausted,
    /// Relational has no snapshot identities left.
    SnapshotIdentityExhausted,
    /// The transaction already holds the maximum number of savepoints.
    SavepointCapacityExhausted {
        /// The savepoint bound.
        maximum_savepoints: usize,
    },
    /// Relational has no savepoint identities left.
    SavepointIdentityExhausted,
    /// Relational already holds the maximum number of candidates.
    CandidateCapacityExhausted {
        /// The candidate bound.
        maximum_candidates: usize,
    },
    /// Relational already holds the maximum number of published snapshot
    /// handles.
    PublishedSnapshotCapacityExhausted {
        /// The published snapshot handle bound.
        maximum_handles: usize,
    },
    /// Relational has no candidate identities left.
    CandidateIdentityExhausted,
    /// Another publication holds the patch-position reservation; retry later.
    PatchPositionReservationContended,
    /// Relational has no proposal identities left.
    ProposalIdentityExhausted,
    /// The provider panicked while executing the invariant.
    ProviderPanicked,
    /// The provider returned evidence bound to a different invariant execution,
    /// or a retained admission was presented for another attempt.
    EvidenceSubstitution,
    /// The provider loaded no state although the plan expected some.
    EmptyStateLoad,
    /// The loaded state omits a fact the plan requires, or includes one outside
    /// the plan's closure.
    StateLoadClosureMismatch,
    /// The verdicts presented do not satisfy the contract, for example a receipt
    /// from another session or too few passed slots.
    VerdictPostureMismatch,
}

/// How to read an invariant execution failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantExecutionFailurePosture {
    /// The attempt was refused on its merits.
    Denied,
    /// A bound or identity space ran out; the attempt may succeed once
    /// capacity returns.
    Exhausted,
}

/// A typed invariant execution failure: its kind, its posture, a detail
/// message, and the custom invariant's own denial when there is one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantExecutionFailure {
    kind: WorthQueryInvariantExecutionDenialKind,
    posture: WorthQueryInvariantExecutionFailurePosture,
    detail: Arc<str>,
    custom_invariant: Option<WorthQueryCustomInvariantDenial>,
    native: Option<NativeDenial>,
}

impl WorthQueryInvariantExecutionFailure {
    /// A failure with the `Denied` posture.
    pub fn new(kind: WorthQueryInvariantExecutionDenialKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            posture: WorthQueryInvariantExecutionFailurePosture::Denied,
            detail: detail.into(),
            custom_invariant: None,
            native: None,
        }
    }

    pub(crate) fn exhausted(
        kind: WorthQueryInvariantExecutionDenialKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            posture: WorthQueryInvariantExecutionFailurePosture::Exhausted,
            detail: detail.into(),
            custom_invariant: None,
            native: None,
        }
    }

    pub(crate) fn custom_invariant(
        custom_invariant: WorthQueryCustomInvariantDenial,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied,
            posture: WorthQueryInvariantExecutionFailurePosture::Denied,
            detail: detail.into(),
            custom_invariant: Some(custom_invariant),
            native: None,
        }
    }

    pub(crate) fn native_staging(
        denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self::native_denied(NativeDenial::Staging(denial), detail)
    }
    pub(crate) fn physical_allocation(
        denial: ExecutionAllocationDenial,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self::native_denied(NativeDenial::Allocation(denial), detail)
    }
    fn native_denied(native: NativeDenial, detail: impl Into<Arc<str>>) -> Self {
        let (kind, posture) = native.classification();
        Self {
            kind,
            posture,
            detail: detail.into(),
            custom_invariant: None,
            native: Some(native),
        }
    }
    /// Exact physical owner cause, including its original checked quote.
    pub fn allocation_denial(&self) -> Option<&ExecutionAllocationDenial> {
        self.native.as_ref().and_then(NativeDenial::allocation)
    }
    /// Original native staging refusal, including input-directory/count failures.
    pub fn relational_staging_denial(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::RelationalTransactionStagingDenial> {
        match self.native.as_ref() {
            Some(NativeDenial::Staging(denial)) => Some(denial),
            _ => None,
        }
    }
    /// Why execution stopped.
    pub fn kind(&self) -> WorthQueryInvariantExecutionDenialKind {
        self.kind
    }

    /// A human-readable detail message.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Whether the failure was a refusal or exhaustion.
    pub fn posture(&self) -> WorthQueryInvariantExecutionFailurePosture {
        self.posture
    }

    /// The custom invariant's own denial, present only for
    /// `CustomInvariantDenied`.
    pub fn custom_invariant_denial(&self) -> Option<&WorthQueryCustomInvariantDenial> {
        self.custom_invariant.as_ref()
    }
}
