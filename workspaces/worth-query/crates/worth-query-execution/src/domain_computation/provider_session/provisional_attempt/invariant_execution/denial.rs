use std::sync::Arc;

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
    /// Validating the candidate would take more work than the validator bound.
    CandidateValidatorWorkExceeded {
        /// The validator's work bound.
        maximum_work: usize,
        /// The work this candidate needed.
        required_work: usize,
    },
    /// The provider has no port for this kind of invariant execution.
    ProviderUnsupported,
    /// The provider, or Relational behind it, refused the load, the touches, or
    /// the verdict. The failure's detail says which.
    ProviderRejected,
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
    /// The transaction overlay would exceed its byte bound.
    TransactionOverlayCapacityExhausted {
        /// The byte bound.
        maximum_bytes: u64,
        /// The bytes this attempt needed.
        required_bytes: u64,
    },
    /// The transaction footprint would exceed its locus bound.
    TransactionFootprintCapacityExhausted {
        /// The locus bound.
        maximum_loci: usize,
        /// The loci this attempt needed.
        required_loci: usize,
    },
    /// The transaction already holds the maximum number of savepoints.
    SavepointCapacityExhausted {
        /// The savepoint bound.
        maximum_savepoints: usize,
    },
    /// A savepoint's footprint would exceed its locus bound.
    SavepointFootprintCapacityExhausted {
        /// The locus bound.
        maximum_loci: usize,
        /// The loci this attempt needed.
        required_loci: usize,
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
    /// The prepared root would exceed its byte budget.
    PreparedRootBudgetExhausted {
        /// The byte bound.
        maximum_bytes: u64,
        /// The bytes this attempt needed.
        required_bytes: u64,
    },
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
}

impl WorthQueryInvariantExecutionFailure {
    /// A failure with the `Denied` posture.
    pub fn new(kind: WorthQueryInvariantExecutionDenialKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            posture: WorthQueryInvariantExecutionFailurePosture::Denied,
            detail: detail.into(),
            custom_invariant: None,
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
