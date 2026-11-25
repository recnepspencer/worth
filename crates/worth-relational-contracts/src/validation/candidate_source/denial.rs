//! Original preparation failures survive the Native/Query handoff.
use crate::derived_relation::read_denial::DerivedRelationReadDenial;
use crate::internal_port::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;
use crate::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::mvcc::publication::companion::{
    preflight_stop::CompanionPreflightStop, registration_stop::PublicationCompanionRegistrationStop,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedIndexSourcePreparationDenial {
    Content(crate::content::denial::RelationalContentDenial),
    Source(DerivedRelationReadDenial),
    Provider(super::provider_failure::ManagedIndexSourceProviderFailure),
    Publication(CompanionPreflightStop),
    Registration(PublicationCompanionRegistrationStop),
    MissingParticipant,
    IncompletePreparation,
    ForeignSource,
    RootUnavailable,
    PredecessorUnavailable {
        predecessor: crate::identity::data::EntityId,
    },
    PublicCacheIdentityExhausted,
}

impl From<DerivedRelationReadDenial> for ManagedIndexSourcePreparationDenial {
    fn from(cause: DerivedRelationReadDenial) -> Self {
        Self::Source(cause)
    }
}

impl ManagedIndexSourcePreparationDenial {
    /// This exact Source posture had no available root; never means empty rows.
    pub fn source_unavailable(&self) -> bool {
        matches!(
            self,
            Self::RootUnavailable | Self::PredecessorUnavailable { .. }
        )
    }

    /// Borrow the provider's original owned failure without changing its custody.
    pub fn provider_failure(
        &self,
    ) -> Option<&super::provider_failure::ManagedIndexSourceProviderFailure> {
        match self {
            Self::Provider(cause) => Some(cause),
            _ => None,
        }
    }
}

impl From<crate::derived_relation::definition::DerivedRelationDenial>
    for ManagedIndexSourcePreparationDenial
{
    fn from(cause: crate::derived_relation::definition::DerivedRelationDenial) -> Self {
        Self::Source(cause.into())
    }
}

/// Operations used by the Relational owners.
pub trait ManagedIndexSourcePreparationDenialOwnerPort: Sized {
    fn into_commit_error(
        self,
        branch: crate::history::data::coordinates::BranchId,
        commit: Option<u64>,
        version: crate::identity::data::VersionId,
    ) -> crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
    fn execution_cause(
        &self,
    ) -> Option<crate::execution::denial_cause::RelationalExecutionDenialCause>;
}

impl ManagedIndexSourcePreparationDenialOwnerPort for ManagedIndexSourcePreparationDenial {
    fn into_commit_error(
        self,
        branch: crate::history::data::coordinates::BranchId,
        commit: Option<u64>,
        version: crate::identity::data::VersionId,
    ) -> crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
        use crate::transactions::data::outcomes::{
            commit_preparation::CommitPreparationError,
            transaction_commit_error::{
                CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
            },
        };
        if let Some(cause) = self.execution_cause() {
            return TransactionCommitError::execution(CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            });
        }
        TransactionCommitError::preparation(CommitPreparationError::managed_index_source(
            branch, commit, version, self,
        ))
    }

    fn execution_cause(
        &self,
    ) -> Option<crate::execution::denial_cause::RelationalExecutionDenialCause> {
        use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
        use crate::validation::custom_rule::interruption::CustomInvariantInterruption;
        use crate::validation::data::structural_walk_denial::StructuralWalkDenialKind;

        match self {
            Self::Content(crate::content::denial::RelationalContentDenial::Execution(cause)) => {
                Some(*cause)
            }
            Self::Source(DerivedRelationReadDenial::Request(cause)) => Some(*cause),
            Self::Source(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Interrupted(interruption),
            )) => Some(match interruption {
                CustomInvariantInterruption::Cancelled => Cause::Cancelled,
                CustomInvariantInterruption::DeadlineElapsed => Cause::DeadlineElapsed,
            }),
            Self::Source(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::MemoryAdmission(cause),
            )) => Some(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(*cause),
            )),
            _ => None,
        }
    }
}
