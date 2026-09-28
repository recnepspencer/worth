/// The specific reason installing or publishing an application's primary graph
/// runtime was refused.
///
/// Families: runtime and schema identity; bootstrap data; Relational storage
/// capacity and identity exhaustion; index, schema, policy, and Bridge
/// rejection; and the inventories of handlers, invariant factories, producers,
/// conditional bindings, and managed computation owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrimaryGraphInstallationDenialKind {
    /// A binding or input belongs to a different runtime.
    ForeignRuntime,
    /// The application's contributions differ from the installed contribution
    /// catalog.
    ContributionInventoryMismatch,
    /// A binding names a member its contribution does not own.
    ContributionMemberMismatch,
    /// The installed schema changed since it was presented.
    StaleInstalledSchema,
    /// The primary graph is already installed.
    AlreadyInstalled,
    /// A binding the declaration requires is not installed.
    BindingNotInstalled,
    /// A binding belongs to a different schema.
    BindingSchemaMismatch,
    /// Two bootstrap principals share an external identity.
    DuplicateExternalIdentity,
    /// Two bootstrap principals share a principal identity.
    DuplicatePrincipalIdentity,
    /// Two bootstrap principals share a principal key.
    DuplicatePrincipalKey,
    /// The bootstrap has no content.
    EmptyBootstrap,
    /// A schema member is not valid for installation.
    InvalidSchemaMember,
    /// Relational rejected the installation commit.
    RelationalCommitRejected,
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The retention owner for the initial schema is unavailable.
    RetentionOwnerUnavailable,
    /// The initial schema retains more roots than allowed.
    RetentionRootSetTooLarge,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The transaction overlay would exceed its byte limit.
    TransactionOverlayCapacityExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    /// The transaction footprint would exceed its locus limit.
    TransactionFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    /// The savepoint limit was reached.
    SavepointCapacityExhausted { maximum_savepoints: usize },
    /// A savepoint footprint would exceed its locus limit.
    SavepointFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    /// The runtime ran out of savepoint identities.
    SavepointIdentityExhausted,
    /// The provider's candidate limit was reached.
    CandidateCapacityExhausted { maximum_candidates: usize },
    /// The provider's published-snapshot limit was reached.
    PublishedSnapshotCapacityExhausted { maximum_handles: usize },
    /// The runtime ran out of candidate identities.
    CandidateIdentityExhausted,
    /// The prepared root would exceed its byte budget.
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    /// The commit position was contended.
    PatchPositionReservationContended,
    /// The runtime ran out of proposal identities.
    ProposalIdentityExhausted,
    /// Building an installed index was rejected.
    IndexBuildRejected,
    /// Relational rejected the schema.
    RelationalSchemaRejected,
    /// The Relational runtime was already published.
    RelationalRuntimeAlreadyPublished,
    /// The installed authorization policy was rejected.
    AuthorizationPolicyRejected,
    /// The Runtime Bridge rejected the installation.
    RuntimeBridgeRejected,
    /// The schema declares conditional nodes, so the runtime must be published
    /// through conditional installation.
    ConditionalBindingsRequired,
    /// A declared operation has no mutation handler.
    MissingMutationHandler,
    /// An operation has more than one mutation handler.
    DuplicateMutationHandler,
    /// A mutation handler belongs to a different application.
    ForeignMutationHandler,
    /// A mutation handler does not match its operation's declared meaning.
    MutationHandlerMeaningMismatch,
    /// A handler was offered for a workflow control binding, which the workflow
    /// kernel records itself.
    WorkflowControlHandler,
    /// A declared invariant has no factory.
    MissingInvariantFactory,
    /// An invariant has more than one factory.
    DuplicateInvariantFactory,
    /// An invariant factory belongs to a different application.
    ForeignInvariantFactory,
    /// An invariant factory does not match its invariant's declared meaning.
    InvariantFactoryMeaningMismatch,
    /// An invariant factory rejected installation.
    InvariantFactoryRejected,
    /// A program-rostered rule's declared work budget cannot carry the platform
    /// reserve its branch program activation read needs.
    ProgramActivationWorkReserveOverflow,
    /// The invariant installation receipt does not match this installation.
    InvariantInstallationReceiptMismatch,
    /// Recovery from a checkpoint was rejected.
    CheckpointRecoveryRejected,
    /// A declared output producer has no binding.
    MissingProducerBinding,
    /// A producer is bound more than once.
    DuplicateProducerBinding,
    /// A producer operation is bound more than once.
    DuplicateProducerOperationBinding,
    /// A producer binding has no provider.
    MissingProducerProvider,
    /// A producer binding belongs to a different application.
    ForeignProducerBinding,
    /// A producer binding does not match its declared meaning.
    ProducerBindingMeaningMismatch,
    /// A declared conditional operation has no binding.
    MissingConditionalBinding,
    /// A conditional operation is bound more than once.
    DuplicateConditionalBinding,
    /// An output case has no applicable producer.
    MissingApplicableProducer,
    /// More than one producer applies to the same output case.
    AmbiguousApplicableProducer,
    /// A declared managed computation has no owner.
    MissingManagedComputationOwner,
    /// A managed computation has more than one owner.
    DuplicateManagedComputationOwner,
    /// A managed computation owner belongs to a different application.
    ForeignManagedComputationOwner,
    /// A managed computation owner does not match its declared meaning.
    ManagedComputationOwnerMeaningMismatch,
}

/// Refusal to install or publish an application's primary graph runtime.
///
/// No runtime was published. [`Self::kind`] says why and [`Self::subject`]
/// names what was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphInstallationDenial {
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: String,
}

impl WorthQueryPrimaryGraphInstallationDenial {
    pub(super) fn new(
        kind: WorthQueryPrimaryGraphInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    /// Maps failure to resolve a declaration-required installed binding while
    /// constructing an unpublished application root.
    #[doc(hidden)]
    pub fn binding_not_installed(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryPrimaryGraphInstallationDenialKind::BindingNotInstalled,
            subject,
        )
    }

    pub const fn kind(&self) -> WorthQueryPrimaryGraphInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryPrimaryGraphInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "primary graph installation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryPrimaryGraphInstallationDenial {}
