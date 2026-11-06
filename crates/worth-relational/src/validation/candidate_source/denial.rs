//! Original preparation failures survive the Native/Query handoff.
use crate::derived_relation::DerivedRelationReadDenial;
use crate::mvcc::{CompanionPreflightStop, PublicationCompanionRegistrationStop};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedIndexSourcePreparationDenial {
    Content(crate::content::RelationalContentDenial),
    Source(DerivedRelationReadDenial),
    Provider(super::ManagedIndexSourceProviderFailure),
    Publication(CompanionPreflightStop),
    Registration(PublicationCompanionRegistrationStop),
    MissingParticipant,
    IncompletePreparation,
    ForeignSource,
    RootUnavailable,
    PublicCacheIdentityExhausted,
}

impl From<DerivedRelationReadDenial> for ManagedIndexSourcePreparationDenial {
    fn from(cause: DerivedRelationReadDenial) -> Self {
        Self::Source(cause)
    }
}

impl ManagedIndexSourcePreparationDenial {
    pub(crate) fn into_commit_error(
        self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> crate::transactions::data::TransactionCommitError {
        use crate::transactions::data::{
            CommitExecutionDenial, CommitExecutionDenialKind, CommitPreparationError,
            TransactionCommitError,
        };
        if let Some(cause) = self.execution_cause() {
            return TransactionCommitError::execution(CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            });
        }
        TransactionCommitError::preparation(CommitPreparationError::managed_index_source(
            basis, self,
        ))
    }

    /// This exact Source posture had no available root; never means empty rows.
    pub fn source_unavailable(&self) -> bool {
        matches!(self, Self::RootUnavailable)
    }

    /// Borrow the provider's original owned failure without changing its custody.
    pub fn provider_failure(&self) -> Option<&super::ManagedIndexSourceProviderFailure> {
        match self {
            Self::Provider(cause) => Some(cause),
            _ => None,
        }
    }

    pub(super) fn execution_cause(
        &self,
    ) -> Option<crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        use crate::validation::data::{CustomInvariantInterruption, StructuralWalkDenialKind};

        match self {
            Self::Content(crate::content::RelationalContentDenial::Execution(cause)) => {
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

impl From<crate::derived_relation::DerivedRelationDenial> for ManagedIndexSourcePreparationDenial {
    fn from(cause: crate::derived_relation::DerivedRelationDenial) -> Self {
        Self::Source(cause.into())
    }
}
