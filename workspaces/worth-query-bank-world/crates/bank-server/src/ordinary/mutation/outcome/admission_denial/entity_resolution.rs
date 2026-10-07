use worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind as QueryKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEntityResolutionDenialKind {
    Handle(worth_query_host::facade::primary_graph::WorthQueryHandleDenial),
    Cancelled,
    DeadlineExceeded,
    PrimaryGraphNotInstalled,
    FieldNotInstalled,
    ValueEncodingRejected,
    EqualityIndexUnavailable,
    UnknownEntity,
    AmbiguousEntity,
    CorruptIdentityIndex,
    ProjectionWorkBudgetExceeded,
    ProjectionPreparationMemoryExhausted,
    InvalidCandidateLimit,
    CandidateLimitExceeded { maximum: usize },
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignResolutionTruth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankEntityResolutionDenial {
    kind: BankEntityResolutionDenialKind,
}

impl BankEntityResolutionDenial {
    pub const fn kind(self) -> BankEntityResolutionDenialKind {
        self.kind
    }

    pub const fn code(self) -> &'static str {
        use BankEntityResolutionDenialKind as Bank;
        match self.kind {
            Bank::Handle(_) => "closed",
            Bank::Cancelled => "cancelled",
            Bank::DeadlineExceeded => "deadline-exceeded",
            Bank::PrimaryGraphNotInstalled => "primary-graph-not-installed",
            Bank::FieldNotInstalled => "field-not-installed",
            Bank::ValueEncodingRejected => "value-encoding-rejected",
            Bank::EqualityIndexUnavailable => "equality-index-unavailable",
            Bank::UnknownEntity => "unknown-entity",
            Bank::AmbiguousEntity => "ambiguous-entity",
            Bank::CorruptIdentityIndex => "corrupt-identity-index",
            Bank::ProjectionWorkBudgetExceeded => "projection-work-budget-exceeded",
            Bank::ProjectionPreparationMemoryExhausted => "projection-preparation-memory-exhausted",
            Bank::InvalidCandidateLimit => "invalid-candidate-limit",
            Bank::CandidateLimitExceeded { .. } => "candidate-limit-exceeded",
            Bank::ActiveSnapshotCapacityExhausted { .. } => "active-snapshot-capacity-exhausted",
            Bank::SnapshotIdentityExhausted => "snapshot-identity-exhausted",
            Bank::RetentionCapacityExhausted => "retention-capacity-exhausted",
            Bank::RetentionIdentityExhausted => "retention-identity-exhausted",
            Bank::ForeignResolutionTruth => "foreign-resolution-truth",
        }
    }

    pub(crate) const fn from_query(kind: QueryKind) -> Self {
        use BankEntityResolutionDenialKind as Bank;
        let kind = match kind {
            QueryKind::Handle(handle) => Bank::Handle(handle),
            QueryKind::Cancelled => Bank::Cancelled,
            QueryKind::DeadlineExceeded => Bank::DeadlineExceeded,
            QueryKind::PrimaryGraphNotInstalled => Bank::PrimaryGraphNotInstalled,
            QueryKind::FieldNotInstalled => Bank::FieldNotInstalled,
            QueryKind::ValueEncodingRejected => Bank::ValueEncodingRejected,
            QueryKind::EqualityIndexUnavailable => Bank::EqualityIndexUnavailable,
            QueryKind::UnknownEntity => Bank::UnknownEntity,
            QueryKind::AmbiguousEntity => Bank::AmbiguousEntity,
            QueryKind::CorruptIdentityIndex => Bank::CorruptIdentityIndex,
            QueryKind::ProjectionWorkBudgetExceeded => Bank::ProjectionWorkBudgetExceeded,
            QueryKind::ProjectionPreparationMemoryExhausted => {
                Bank::ProjectionPreparationMemoryExhausted
            }
            QueryKind::InvalidCandidateLimit => Bank::InvalidCandidateLimit,
            QueryKind::CandidateLimitExceeded { maximum } => {
                Bank::CandidateLimitExceeded { maximum }
            }
            QueryKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Bank::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            QueryKind::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
            QueryKind::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
            QueryKind::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
            QueryKind::ForeignResolutionTruth => Bank::ForeignResolutionTruth,
        };
        Self { kind }
    }
}

impl std::fmt::Display for BankEntityResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}
