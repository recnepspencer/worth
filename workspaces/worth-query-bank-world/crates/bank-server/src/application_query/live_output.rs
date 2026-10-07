//! Bank-owned descriptions of live Query execution outcomes.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCauseDenialKind, WorthQueryApplicationLiveCloseOutcome,
    WorthQueryApplicationLiveOverflow, WorthQueryApplicationProjectionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankApplicationLiveOverflow {
    missed_commit_batches: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationLiveProjectionDenial {
    FieldNotProjected,
    FieldContractMismatch,
    FieldTypeMismatch,
    FieldOmitted,
    RelationNotProjected,
    RelationContractMismatch,
    RelationCardinalityMismatch,
    RelationOmitted,
    DomainProjectionRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationLiveCauseDenial {
    TargetIdentityUnavailable,
    TargetOutsideScope,
    ResultShapeUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationLiveCloseOutcome {
    Handle(worth_query_host::facade::primary_graph::WorthQueryHandleDenial),
    Completed,
    Unavailable,
}

impl BankApplicationLiveOverflow {
    pub const fn missed_commit_batches(self) -> u64 {
        self.missed_commit_batches
    }

    pub(crate) fn from_query(overflow: WorthQueryApplicationLiveOverflow) -> Self {
        Self {
            missed_commit_batches: overflow.missed_commit_batches(),
        }
    }
}

impl BankApplicationLiveProjectionDenial {
    pub const fn code(self) -> &'static str {
        match self {
            Self::FieldNotProjected => "field-not-projected",
            Self::FieldContractMismatch => "field-contract-mismatch",
            Self::FieldTypeMismatch => "field-type-mismatch",
            Self::FieldOmitted => "field-omitted",
            Self::RelationNotProjected => "relation-not-projected",
            Self::RelationContractMismatch => "relation-contract-mismatch",
            Self::RelationCardinalityMismatch => "relation-cardinality-mismatch",
            Self::RelationOmitted => "relation-omitted",
            Self::DomainProjectionRejected => "domain-projection-rejected",
        }
    }

    pub(crate) const fn from_query(kind: WorthQueryApplicationProjectionDenialKind) -> Self {
        use WorthQueryApplicationProjectionDenialKind as Query;
        match kind {
            Query::FieldNotProjected => Self::FieldNotProjected,
            Query::FieldContractMismatch => Self::FieldContractMismatch,
            Query::FieldTypeMismatch => Self::FieldTypeMismatch,
            Query::FieldOmitted => Self::FieldOmitted,
            Query::RelationNotProjected => Self::RelationNotProjected,
            Query::RelationContractMismatch => Self::RelationContractMismatch,
            Query::RelationCardinalityMismatch => Self::RelationCardinalityMismatch,
            Query::RelationOmitted => Self::RelationOmitted,
            Query::DomainProjectionRejected => Self::DomainProjectionRejected,
        }
    }
}

impl BankApplicationLiveCauseDenial {
    pub const fn code(self) -> &'static str {
        match self {
            Self::TargetIdentityUnavailable => "target-identity-unavailable",
            Self::TargetOutsideScope => "target-outside-scope",
            Self::ResultShapeUnavailable => "result-shape-unavailable",
        }
    }

    pub(crate) const fn from_query(kind: WorthQueryApplicationLiveCauseDenialKind) -> Self {
        use WorthQueryApplicationLiveCauseDenialKind as Query;
        match kind {
            Query::TargetIdentityUnavailable => Self::TargetIdentityUnavailable,
            Query::TargetOutsideScope => Self::TargetOutsideScope,
            Query::ResultShapeUnavailable => Self::ResultShapeUnavailable,
        }
    }
}

impl BankApplicationLiveCloseOutcome {
    pub(crate) fn from_query(outcome: WorthQueryApplicationLiveCloseOutcome) -> Self {
        match outcome {
            WorthQueryApplicationLiveCloseOutcome::Handle(handle) => Self::Handle(handle),
            WorthQueryApplicationLiveCloseOutcome::Completed(_) => Self::Completed,
            WorthQueryApplicationLiveCloseOutcome::Unavailable => Self::Unavailable,
        }
    }
}
