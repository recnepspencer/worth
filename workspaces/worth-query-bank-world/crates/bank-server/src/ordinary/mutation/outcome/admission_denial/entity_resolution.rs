use worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind as QueryKind;
use worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind as Refusal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEntityResolutionDenialKind {
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    ForeignPhase,
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
            Bank::ExecutionRequest(_) => "execution-request",
            Bank::ForeignPhase => "foreign-phase",
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

    pub(crate) fn from_query(kind: QueryKind) -> Self {
        use BankEntityResolutionDenialKind as Bank;
        let kind = match kind {
            QueryKind::Stopped(cause) => {
                use worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind as Control;
                match worth_query_host::facade::primary_graph::relational_execution_kind(
                    cause, None,
                ) {
                    Ok(kind) => Bank::ExecutionRequest(kind),
                    Err(Control::Cancelled) => Bank::Cancelled,
                    Err(Control::TimedOut) => Bank::DeadlineExceeded,
                }
            }
            QueryKind::Refused(kind) => match kind {
                Refusal::ForeignPhase => Bank::ForeignPhase,
                Refusal::PrimaryGraphNotInstalled => Bank::PrimaryGraphNotInstalled,
                Refusal::FieldNotInstalled => Bank::FieldNotInstalled,
                Refusal::ValueEncodingRejected => Bank::ValueEncodingRejected,
                Refusal::EqualityIndexUnavailable => Bank::EqualityIndexUnavailable,
                Refusal::UnknownEntity => Bank::UnknownEntity,
                Refusal::AmbiguousEntity => Bank::AmbiguousEntity,
                Refusal::CorruptIdentityIndex => Bank::CorruptIdentityIndex,
                Refusal::InvalidCandidateLimit => Bank::InvalidCandidateLimit,
                Refusal::CandidateLimitExceeded { maximum } => {
                    Bank::CandidateLimitExceeded { maximum }
                }
                Refusal::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => Bank::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
                Refusal::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
                Refusal::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
                Refusal::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
                Refusal::ForeignResolutionTruth => Bank::ForeignResolutionTruth,
            },
        };
        Self { kind }
    }
}

impl std::fmt::Display for BankEntityResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

#[cfg(test)]
mod stop_code_tests;
