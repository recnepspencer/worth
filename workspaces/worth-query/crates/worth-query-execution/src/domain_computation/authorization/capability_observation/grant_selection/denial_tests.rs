//! Operational refusals must never appear as a policy/observation rejection.
use super::*;
use worth_relational::facade::indexes::BoundedRelationJoinLookupDenialKind as Kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn grant_selection_names_deadlines_and_every_ceiling_as_operational_stops() {
    assert_eq!(
        grant_lookup_denial_kind(Kind::Execution(Cause::Cancelled)),
        WorthQueryOperationAuthorizationDenialKind::Cancelled
    );
    assert_eq!(
        grant_lookup_denial_kind(Kind::Execution(Cause::DeadlineElapsed)),
        WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded
    );
    for cause in [
        Cause::WorkExhausted,
        Cause::WorkLimitExceedsParent,
        Cause::WorkerLimitExceedsParent,
        Cause::MemoryLimitExceedsParent,
        Cause::PolicyMemoryExhausted {
            requested: 2,
            admitted: 1,
            ancestor: 0,
        },
        Cause::ProcessMemoryExhausted {
            requested: 2,
            admitted: 1,
        },
        Cause::DeclaredMemoryExhausted {
            requested: 2,
            admitted: 1,
        },
        Cause::WorkCounterOverflow,
        Cause::ResultCapacityExceeded,
        Cause::ScratchCapacityExceeded,
    ] {
        assert_eq!(
            grant_lookup_denial_kind(Kind::Execution(cause)),
            WorthQueryOperationAuthorizationDenialKind::CanonicalWorkDenied
        );
    }
    assert_eq!(
        grant_lookup_denial_kind(Kind::CorruptIndexEntries),
        WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected
    );
}

#[test]
fn grant_selection_retains_every_structural_execution_cause() {
    for cause in [
        Cause::NoActiveExecutionScope,
        Cause::UnrelatedNestedLease,
        Cause::EquivalenceContractUnavailable,
        Cause::NestedStopped,
        Cause::WorkerFailed,
        Cause::UncheckedCustomKernel,
    ] {
        assert_eq!(
            grant_lookup_denial_kind(Kind::Execution(cause)),
            WorthQueryOperationAuthorizationDenialKind::RelationalExecution(cause)
        );
    }
}
