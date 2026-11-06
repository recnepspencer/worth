//! Moved whole from `domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial`: the items below and every impl they own.

/// Why a commit was refused before publication.
///
/// Every kind is a refusal before the branch moved: nothing was committed.
/// Capacity and identity kinds call for a later retry; binding and lane kinds
/// mean the attempt went through the wrong lane or program and will not succeed
/// as presented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialKind {
    /// Retained content disagreed with its certified identity before publication.
    ContentInconsistency,
    /// Exact Native content refusal, distinct from request/resource stops.
    Content(worth_relational::facade::content::RelationalContentRejection),
    StorageUnavailable(worth_relational::facade::storage::RelationalStorageUnavailable),
    /// The execution owner refused preparation before any commit effect.
    ExecutionResource {
        denial: crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial,
        partition_identity: Option<u64>,
        policy_ancestor: Option<u32>,
    },
    ExecutionNestedPatternStopped { partition_identity: Option<u64> },
    ExecutionWorkerPanicked { partition_identity: Option<u64> },
    ExecutionUncheckedCustomKernel { partition_identity: Option<u64> },
    ExecutionIdentitiesNotCanonical { partition_identity: Option<u64> },
    /// The owner refused the attempt at the denial's stage. Execution evidence
    /// is available through `WorthQueryApplicationCommitDenial::execution_denial_cause`.
    /// Its category and stage do not determine the evidence's identity.
    ProviderRejected,
    /// An installed custom invariant refused the candidate; see
    /// `custom_invariant_denial()`.
    CustomInvariantDenied,
    /// A domain field could not prove its emitted bytes readable before publication.
    DomainValueRejected,

    /// The workflow step this commit carries could not be settled; `kind` says why.
    WorkflowSettlementDenied {
        kind: crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind,
    },
    /// The product basis the attempt relied on was no longer current.
    ProductBasisStale,
    /// A write of a unique field names a value another live entity holds, or
    /// the program writes one value twice; `detail()` names the field.
    UniqueValueTaken,
    /// An existing field write lacks its exact completed decision observation.
    UnobservedWrite,
    /// A unique field's equality index could not answer the lookup its write
    /// must observe.
    UniqueIndexUnavailable,
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
    CorrectionContextMismatch,
    /// The actual inverse output does not preserve its installed authored family correspondence.
    CorrectionIdentityMismatch,
    /// The installed operation owns an executable body, but the candidate was
    /// not produced by its native executor. Caller-authored effects cannot
    /// substitute another implementation, even within the declared footprint.
    OperationBodyExecutionRequired,
    /// A signed creation lacks its native authentication or its immutable subject was changed.
    SignedCreationAuthentication { kind: worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial },
    SignedEntityImmutable,
    SigningLifecycleForgery,
    SigningLifecycleUnavailable,
    SigningLifecycleEvidenceCapacity { maximum_evidence: usize },
    SigningLifecycleSubjectCapacity { maximum_subjects: usize },
    SigningLifecycleRepresentationCapacity,
    SigningLifecycleObservation { kind: crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind },
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
