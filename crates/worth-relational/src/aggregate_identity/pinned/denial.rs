//! Typed read-owned declines stay distinct from original execution refusal.
use super::*;
use crate::execution::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinnedAggregateReadDenialKind {
    Read(RelationalExecutionDenialCause),
    NotAggregateRelation,
    AmbiguousAggregateRelation,
    DefinitionUnavailable,
    IdentityUnavailable,
    AmbiguousIdentity,
    LineageCycle,
    MemberUnavailable,
    WrongMemberKind,
    WorkLimitExceeded,
    InvalidStoredAggregate,
    InvalidStoredLineage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PinnedAggregateReadDenial {
    pub kind: PinnedAggregateReadDenialKind,
    pub reads: AggregateReadObservations,
    pub work: u64,
}

pub(super) fn kind(failure: FundingFailure) -> PinnedAggregateReadDenialKind {
    use PinnedAggregateReadDenialKind as Kind;
    match failure {
        FundingFailure::Execution(cause) | FundingFailure::Domain(PlanDenial::Execution(cause)) => {
            Kind::Read(cause)
        }
        FundingFailure::Domain(denial) => match denial {
            PlanDenial::NotAggregateRelation => Kind::NotAggregateRelation,
            PlanDenial::AmbiguousAggregateRelation => Kind::AmbiguousAggregateRelation,
            PlanDenial::DefinitionUnavailable => Kind::DefinitionUnavailable,
            PlanDenial::IdentityUnavailable => Kind::IdentityUnavailable,
            PlanDenial::AmbiguousIdentity => Kind::AmbiguousIdentity,
            PlanDenial::LineageCycle => Kind::LineageCycle,
            PlanDenial::MemberUnavailable => Kind::MemberUnavailable,
            PlanDenial::WrongMemberKind => Kind::WrongMemberKind,
            PlanDenial::ReadWorkLimitExceeded => Kind::WorkLimitExceeded,
            PlanDenial::InvalidStoredLineage => Kind::InvalidStoredLineage,
            _ => Kind::InvalidStoredAggregate,
        },
    }
}

impl std::fmt::Display for PinnedAggregateReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pinned aggregate read refused: {:?}", self.kind)
    }
}
impl std::error::Error for PinnedAggregateReadDenial {}
