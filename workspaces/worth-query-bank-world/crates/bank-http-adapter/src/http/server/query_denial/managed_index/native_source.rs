//! HTTP projection of Native Source preparation and its original walk refusal.
use super::*;
use worth_query_host::facade::primary_graph::{
    DerivedRelationReadDenial, ManagedIndexSourcePreparationDenial, StructuralReadError,
    StructuralWalkDenialKind,
};
pub(super) fn source(cause: &ManagedIndexSourcePreparationDenial) -> BankHttpDenial {
    use ManagedIndexSourcePreparationDenial as Source;
    match cause {
        Source::Content(cause) => crate::http::server::relational_execution_denial::content(*cause),
        Source::Source(DerivedRelationReadDenial::Request(cause)) => {
            crate::http::server::relational_execution_denial::denial(*cause)
        }
        Source::Source(DerivedRelationReadDenial::Structural(cause)) => walk(*cause),
        Source::Source(DerivedRelationReadDenial::Definition(_)) | Source::Provider(_) => {
            unavailable()
        }
        Source::Publication(cause) => {
            crate::http::server::relational_execution_denial::companion(cause.clone())
        }
        Source::MissingParticipant | Source::IncompletePreparation | Source::RootUnavailable => {
            unavailable()
        }
        Source::Registration(_) | Source::ForeignSource | Source::PublicCacheIdentityExhausted => {
            internal_denied()
        }
    }
}
pub(super) fn walk(cause: StructuralWalkDenialKind) -> BankHttpDenial {
    use worth_query_host::facade::{
        application_invariants::CustomInvariantInterruption as Interrupted,
        runtime::{LeaseDenial, WorkCeilingDenial},
    };
    match cause {
        StructuralWalkDenialKind::Interrupted(Interrupted::Cancelled) => cancelled(),
        StructuralWalkDenialKind::Interrupted(Interrupted::DeadlineElapsed) => deadline(),
        StructuralWalkDenialKind::MemoryBudgetExceeded
        | StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded) => exhausted(),
        StructuralWalkDenialKind::MemoryAdmission(cause) => {
            crate::http::server::advancement_denial::request(WorkCeilingDenial::Admission(
                LeaseDenial::MemoryExhausted(cause),
            ))
        }
        StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess) => {
            internal_denied()
        }
        StructuralWalkDenialKind::Read(StructuralReadError::RecordUnavailable) => unavailable(),
    }
}
