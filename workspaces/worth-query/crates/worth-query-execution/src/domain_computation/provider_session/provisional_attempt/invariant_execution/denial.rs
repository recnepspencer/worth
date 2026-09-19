use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryCustomInvariantDenial {
    Violation {
        identity: worth_relational::facade::transactions::CustomInvariantSemanticIdentity,
    },
    Failure {
        identity: worth_relational::facade::transactions::CustomInvariantFailureIdentity,
        phase: worth_relational::facade::transactions::CustomInvariantFailurePhase,
        failure: worth_relational::facade::transactions::ResultCustomInvariantFailureKind,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantExecutionDenialKind {
    InvariantNotInstalled,
    ExecutorRoleMismatch,
    EmptyStateLoadPlan,
    UndeclaredStateLoadFamily,
    StateLoadBudgetExceeded,
    ExecutionBudgetExceeded,
    CandidateValidatorWorkExceeded {
        maximum_work: usize,
        required_work: usize,
    },
    ProviderUnsupported,
    ProviderRejected,
    CustomInvariantDenied,
    ProductBasisStale,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    TransactionOverlayCapacityExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    TransactionFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointCapacityExhausted {
        maximum_savepoints: usize,
    },
    SavepointFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointIdentityExhausted,
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    CandidateIdentityExhausted,
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    PatchPositionReservationContended,
    ProposalIdentityExhausted,
    ProviderPanicked,
    EvidenceSubstitution,
    EmptyStateLoad,
    StateLoadClosureMismatch,
    VerdictPostureMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantExecutionFailurePosture {
    Denied,
    Exhausted,
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantExecutionFailure {
    kind: WorthQueryInvariantExecutionDenialKind,
    posture: WorthQueryInvariantExecutionFailurePosture,
    detail: Arc<str>,
    custom_invariant: Option<WorthQueryCustomInvariantDenial>,
}

impl WorthQueryInvariantExecutionFailure {
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

    pub fn kind(&self) -> WorthQueryInvariantExecutionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn posture(&self) -> WorthQueryInvariantExecutionFailurePosture {
        self.posture
    }

    pub fn custom_invariant_denial(&self) -> Option<&WorthQueryCustomInvariantDenial> {
        self.custom_invariant.as_ref()
    }
}
