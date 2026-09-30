//! Pre-publication application denial categories and owner evidence.

mod capacity;
mod program_binding;
mod recorded_idempotency;
mod workflow;

/// Why a commit was refused before publication.
///
/// Every kind is a refusal before the branch moved: nothing was committed.
/// Capacity and identity kinds call for a later retry; binding and lane kinds
/// mean the attempt went through the wrong lane or program and will not succeed
/// as presented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialKind {
    /// The owner refused the attempt at the denial's stage; `detail()` may say why.
    ProviderRejected,
    /// An installed custom invariant refused the candidate; see
    /// `custom_invariant_denial()`.
    CustomInvariantDenied,
    /// Validating the candidate needed more work than its budget allows.
    CandidateValidatorWorkExceeded {
        maximum_work: usize,
        required_work: usize,
    },
    /// The workflow step this commit carries could not be settled; `kind` says why.
    WorkflowSettlementDenied {
        kind: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind,
    },
    /// The product basis the attempt relied on was no longer current.
    ProductBasisStale,
    /// The owner already held its maximum number of active snapshots.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    /// The owner had no room to retain another basis.
    RetentionCapacityExhausted,
    /// The owner ran out of identities for retained bases.
    RetentionIdentityExhausted,
    /// The owner ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The owner ran out of candidate identities.
    CandidateIdentityExhausted,
    /// The prepared candidate would exceed its byte budget.
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    /// Maintaining indexes for the candidate exceeded its work budget.
    IndexMaintenanceBudgetExceeded,
    /// The owner ran out of index generation identities.
    IndexGenerationIdentityExhausted,
    /// The idempotency key is already bound to a different intent. New intent needs
    /// a new key.
    IdempotencyIntentDrift,
    /// The key records `commit` for this intent, but this runtime does not
    /// retain its receipt: the commit was performed before a restore or
    /// reopen, or by a product occurrence that has since retired. The commit
    /// stands and nothing commits again; read current state to observe it.
    IdempotencyReceiptNotRetained {
        commit: worth_relational::facade::history::CommitId,
    },
    /// The key's record predates the durable intent encoding. Its durable
    /// parts match, but it names its operation only by the admitting
    /// installation's seal, which no later runtime can confirm. Nothing is
    /// committed; a new request needs a new key.
    IdempotencyIntentUnverifiable,
    /// The idempotency binding does not name the mutation binding whose handler
    /// produced this program. That is a programming error in the caller, not
    /// intent drift: build the binding with `for_mutation_identities` from
    /// identities encoded for the same binding.
    MutationBindingMismatch,
    /// The idempotency binding names the right mutation binding but derives its
    /// intent from a different input than the handler decided on. That is a
    /// programming error in the caller, not intent drift: build the binding from
    /// the same key and input the handler ran with.
    MutationInputMismatch,
    /// The operation is bound to an elevation lifecycle and must be committed
    /// through its elevation lane, not as a plain commit.
    ElevationTransitionRequired,
    /// The effect program presented as an elevation request does not carry the
    /// effects its elevation binding requires.
    ElevationRequestProgramMismatch,
    /// The effect program presented as an elevation approval does not carry the
    /// effects its elevation binding requires.
    ElevationApprovalProgramMismatch,
    /// The effect program presented as an elevation close does not carry the effects
    /// its elevation binding requires.
    ElevationCloseProgramMismatch,
    /// The effect program presented as a mandatory review does not carry the effects
    /// its binding requires.
    MandatoryReviewProgramMismatch,
    /// The operation's execution posture requires delegation activation, which a
    /// plain commit cannot perform.
    DelegationActivationRequired,
    /// The operation's execution posture requires capability revocation, which a
    /// plain commit cannot perform.
    CapabilityRevocationRequired,
    /// The operation must run under the branch's program; use a program lane.
    ApplicationProgramRequired,
    /// The operation is guarded by a workflow, so only the workflow runtime may
    /// commit it.
    WorkflowAuthorityRequired,
    /// The presented program is not the one this occurrence runs, which is `active`.
    ProgramNotActiveOnOccurrence {
        active: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    },
    /// This host retired, or is retiring, support for the presented revision.
    ProgramSupportRetired,
    /// This occurrence carries no branch program activation the host can
    /// attribute to an admitted rostered program, so no program-gated commit
    /// can be compared against one.
    ProgramActivationUnresolved,
}

/// Which fail-closed integrity path refused to attribute one occurrence's
/// branch program activation.
///
/// These are distinct causes for the same refusal: a host that never seeded
/// activation, a snapshot that cannot produce the record, and a record naming
/// meaning this host never admitted are three different operator situations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProgramActivationUnresolved {
    /// The branch never published an activation record at all.
    NeverPublished,
    /// The activation record exists but this attempt's snapshot cannot read it.
    Unreadable,
    /// The activation record renders a program this host never rostered.
    NamesNoRosteredProgram,
}

impl WorthQueryProgramActivationUnresolved {
    const fn detail(self) -> &'static str {
        match self {
            Self::NeverPublished => "branch program activation was never published",
            Self::Unreadable => "branch program activation is unreadable on this occurrence",
            Self::NamesNoRosteredProgram => "branch program activation names no rostered program",
        }
    }
}

/// The commit stage at which a denial was decided, for diagnostics.
///
/// The stage locates the refusal; the denial kind says what it was.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialStage {
    ProposalBinding,
    BridgePlanning,
    BasisAdmission,
    ResourceAdmission,
    ManagedRunAdmission,
    ProviderPlan,
    Idempotency,
    DecisionReadSet,
    EffectLowering,
    ElevationTransition,
    DelegationTransition,
    ProvisionalState,
    InvariantExecution,
    ProviderCommit,
}

/// A typed refusal of a commit before publication, carried by the `Denied`
/// commit outcome.
///
/// Nothing was committed. Match on [`kind`](Self::kind); the stage and detail
/// locate the refusal, and the custom-invariant accessors explain a
/// `CustomInvariantDenied` kind.
#[derive(Debug)]
pub struct WorthQueryApplicationCommitDenial {
    kind: WorthQueryApplicationCommitDenialKind,
    stage: WorthQueryApplicationCommitDenialStage,
    detail: Option<std::sync::Arc<str>>,
    custom_invariant: Option<crate::domain_computation::WorthQueryCustomInvariantDenial>,
}

impl WorthQueryApplicationCommitDenial {
    pub const fn kind(&self) -> WorthQueryApplicationCommitDenialKind {
        self.kind
    }

    pub const fn stage(&self) -> WorthQueryApplicationCommitDenialStage {
        self.stage
    }

    /// Returns provider-owned diagnostic detail when the rejection boundary
    /// supplies a concrete cause.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn custom_invariant_denial(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryCustomInvariantDenial> {
        self.custom_invariant.as_ref()
    }

    pub fn custom_invariant_violation_identity(
        &self,
    ) -> Option<&worth_relational::facade::transactions::CustomInvariantSemanticIdentity> {
        match self.custom_invariant.as_ref()? {
            crate::domain_computation::WorthQueryCustomInvariantDenial::Violation { identity } => {
                Some(identity)
            }
            crate::domain_computation::WorthQueryCustomInvariantDenial::Failure { .. } => None,
        }
    }

    pub fn custom_invariant_failure(
        &self,
    ) -> Option<(
        &worth_relational::facade::transactions::CustomInvariantFailureIdentity,
        worth_relational::facade::transactions::CustomInvariantFailurePhase,
        worth_relational::facade::transactions::ResultCustomInvariantFailureKind,
    )> {
        match self.custom_invariant.as_ref()? {
            crate::domain_computation::WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase,
                failure,
            } => Some((identity, *phase, *failure)),
            crate::domain_computation::WorthQueryCustomInvariantDenial::Violation { .. } => None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn custom_invariant_denied(
        stage: WorthQueryApplicationCommitDenialStage,
        custom_invariant: crate::domain_computation::WorthQueryCustomInvariantDenial,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CustomInvariantDenied,
            stage,
            detail: Some(detail.into()),
            custom_invariant: Some(custom_invariant),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn provider_rejected(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_rejected_with_detail(
        stage: WorthQueryApplicationCommitDenialStage,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: Some(detail.into()),
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn product_basis_stale(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_intent_drift(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mutation_binding_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MutationBindingMismatch,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mutation_input_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MutationInputMismatch,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_transition_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn delegation_activation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::DelegationActivationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn capability_revocation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CapabilityRevocationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation) const fn application_program_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            custom_invariant: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_request_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationRequestProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_approval_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationApprovalProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_close_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationCloseProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mandatory_review_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MandatoryReviewProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }
}
