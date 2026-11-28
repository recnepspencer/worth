//! Typed funding outcomes remain intact until their audience adapter consumes them.
use crate::contracts::derived_relation::laws::violation::DerivedRelationLawViolationOwnerPort as _;
use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use worth_execution::{ChargedBytes, MapKernelFailure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::internal_port::aggregate_identity) enum PlanDenial {
    Execution(Cause),
    OrdinaryMetadataWrite,
    CurrentMetadataWrite {
        kind: crate::contracts::identity::data::primitives::KindId,
        record: Option<crate::contracts::transactions::data::primitives::RecordRef>,
    },
    InvalidPermutation,
    InvalidComponentGraph,
    InvalidStoredIdentity,
    IncompleteStoredMembership,
    InvalidStoredLineage,
    NotAggregateRelation,
    AmbiguousAggregateRelation,
    DefinitionUnavailable,
    IdentityUnavailable,
    PublicationRequired,
    AmbiguousIdentity,
    LineageCycle,
    MemberUnavailable,
    WrongMemberKind,
    ReadWorkLimitExceeded,
    Transition(super::super::AggregateTransitionDenial),
    DerivedRead(crate::derived_relation::DerivedRelationReadDenial),
    DerivedLaw {
        definition: crate::derived_relation::DerivedRelationIdentity,
        violation: Box<crate::derived_relation::DerivedRelationLawViolation>,
    },
}
impl From<super::super::AggregateTransitionDenial> for PlanDenial {
    fn from(value: super::super::AggregateTransitionDenial) -> Self {
        Self::Transition(value)
    }
}
impl ChargedBytes for PlanDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::DerivedLaw {
                definition,
                violation,
            } => (definition.schema.0.capacity() as u64)
                .saturating_add(definition.name.capacity() as u64)
                .saturating_add(violation.owned_bytes()),
            Self::DerivedRead(crate::derived_relation::DerivedRelationReadDenial::Definition(
                crate::derived_relation::DerivedRelationDenial::DuplicateDefinition(identity)
                | crate::derived_relation::DerivedRelationDenial::MissingDefinition(identity),
            )) => (identity.schema.0.capacity() as u64)
                .saturating_add(identity.name.capacity() as u64),
            _ => 0,
        }
    }
}
impl From<super::super::AggregateTransitionDenial> for MapKernelFailure<PlanDenial> {
    fn from(value: super::super::AggregateTransitionDenial) -> Self {
        Self::Domain(value.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::internal_port::aggregate_identity) enum FundingFailure {
    Domain(PlanDenial),
    Execution(Cause),
}
impl FundingFailure {
    pub(in crate::internal_port::aggregate_identity) fn into_commit(
        self,
    ) -> TransactionCommitError {
        let cause = match self {
            Self::Domain(PlanDenial::CurrentMetadataWrite { kind, record }) => {
                return TransactionCommitError::publication(
                    crate::contracts::publication::data::publication_error::PublicationError::ordinary_metadata_write(
                        kind, record,
                    ),
                );
            }
            Self::Domain(PlanDenial::DerivedLaw {
                definition,
                violation,
            }) => {
                use crate::contracts::validation::data::results::InvariantViolationFields;
                return TransactionCommitError::conflict(
                    crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                        crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass::InvariantViolation {
                            code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::InvariantViolation,
                            detail: format!(
                                "derived relation '{}' violated {}",
                                definition.name,
                                violation.law.as_str()
                            ),
                            fields: InvariantViolationFields::DerivedRelationGraph {
                                definition,
                                violation: *violation,
                            },
                        },
                    ),
                );
            }
            Self::Domain(PlanDenial::DerivedRead(
                crate::derived_relation::DerivedRelationReadDenial::Request(cause),
            )) => cause,
            Self::Execution(cause) | Self::Domain(PlanDenial::Execution(cause)) => cause,
            Self::Domain(denial) => {
                let detail = match denial {
                    PlanDenial::Transition(denial) => {
                        format!("Native aggregate identity transition denied: {denial:?}")
                    }
                    other => format!("Native aggregate identity preparation denied: {other:?}"),
                };
                return TransactionCommitError::conflict(
                    crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                        crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass::KindSchemaMismatch { detail },
                    ),
                );
            }
        };
        TransactionCommitError::execution(crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
            kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause),
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
    FundingFailure::Execution(crate::contracts::execution::denial::lease_denial(denial))
}
