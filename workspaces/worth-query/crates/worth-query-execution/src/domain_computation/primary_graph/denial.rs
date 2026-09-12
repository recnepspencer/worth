#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrimaryGraphInstallationDenialKind {
    ForeignRuntime,
    ContributionInventoryMismatch,
    ContributionMemberMismatch,
    StaleInstalledSchema,
    AlreadyInstalled,
    BindingNotInstalled,
    BindingSchemaMismatch,
    DuplicateExternalIdentity,
    DuplicatePrincipalIdentity,
    DuplicatePrincipalKey,
    EmptyBootstrap,
    InvalidSchemaMember,
    RelationalCommitRejected,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwnerUnavailable,
    RetentionRootSetTooLarge,
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
    IndexBuildRejected,
    RelationalSchemaRejected,
    RelationalRuntimeAlreadyPublished,
    AuthorizationPolicyRejected,
    RuntimeBridgeRejected,
    ConditionalBindingsRequired,
    MissingMutationHandler,
    DuplicateMutationHandler,
    ForeignMutationHandler,
    MutationHandlerMeaningMismatch,
    MissingInvariantFactory,
    DuplicateInvariantFactory,
    ForeignInvariantFactory,
    InvariantFactoryMeaningMismatch,
    InvariantFactoryRejected,
    InvariantInstallationReceiptMismatch,
}

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
