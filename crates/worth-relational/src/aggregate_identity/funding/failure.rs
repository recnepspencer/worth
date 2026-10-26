//! Typed funding outcomes remain intact until their audience adapter consumes them.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ChargedBytes, MapKernelFailure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::aggregate_identity) enum PlanDenial {
    Execution(Cause),
    OrdinaryMetadataWrite,
    InvalidPermutation,
    InvalidStoredIdentity,
    IncompleteStoredMembership,
    InvalidStoredLineage,
    NotAggregateRelation,
    AmbiguousAggregateRelation,
    DefinitionUnavailable,
    IdentityUnavailable,
    AmbiguousIdentity,
    LineageCycle,
    MemberUnavailable,
    WrongMemberKind,
    ReadWorkLimitExceeded,
    Transition(super::super::AggregateTransitionDenial),
}
impl From<super::super::AggregateTransitionDenial> for PlanDenial {
    fn from(value: super::super::AggregateTransitionDenial) -> Self {
        Self::Transition(value)
    }
}
impl ChargedBytes for PlanDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl From<super::super::AggregateTransitionDenial> for MapKernelFailure<PlanDenial> {
    fn from(value: super::super::AggregateTransitionDenial) -> Self {
        Self::Domain(value.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::aggregate_identity) enum FundingFailure {
    Domain(PlanDenial),
    Execution(Cause),
}
impl FundingFailure {
    pub(in crate::aggregate_identity) fn into_commit(self) -> TransactionCommitError {
        let cause = match self {
            Self::Execution(cause) | Self::Domain(PlanDenial::Execution(cause)) => cause,
            Self::Domain(denial) => {
                let detail = match denial {
                    PlanDenial::Transition(denial) => {
                        format!("Native aggregate identity transition denied: {denial:?}")
                    }
                    other => format!("Native aggregate identity preparation denied: {other:?}"),
                };
                return TransactionCommitError::conflict(
                    crate::transactions::data::CommitConflict::new(
                        crate::transactions::data::ConflictClass::KindSchemaMismatch { detail },
                    ),
                );
            }
        };
        TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        })
    }
}

pub(super) fn kernel(cause: MapKernelFailure<PlanDenial>) -> FundingFailure {
    match cause {
        MapKernelFailure::Domain(PlanDenial::Execution(cause)) => FundingFailure::Execution(cause),
        MapKernelFailure::Domain(denial) => FundingFailure::Domain(denial),
        MapKernelFailure::Stop(stop) => FundingFailure::Execution(stop.into()),
        MapKernelFailure::ResultCapacityExceeded => {
            FundingFailure::Execution(Cause::ResultCapacityExceeded)
        }
        MapKernelFailure::Panic => FundingFailure::Execution(Cause::WorkerFailed),
    }
}
pub(super) fn lease(denial: worth_execution::LeaseDenial) -> FundingFailure {
    FundingFailure::Execution(crate::execution::denial::lease_denial(denial))
}
