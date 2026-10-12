//! Grant lookup stops retain their exact controls, resource kind and memory values.
use super::*;
use worth_relational::facade::indexes::BoundedRelationJoinLookupDenialKind as Kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn grant_selection_names_deadlines_and_every_ceiling_as_operational_stops() {
    use crate::domain_computation::primary_graph::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
    };
    use crate::domain_computation::{
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Session,
    };
    use WorthQueryOperationAuthorizationDenialKind as Denial;
    assert_eq!(
        grant_lookup_denial_kind(Kind::Execution(Cause::Cancelled)),
        Denial::Stopped(Err(Control::Cancelled))
    );
    assert_eq!(
        grant_lookup_denial_kind(Kind::Execution(Cause::DeadlineElapsed)),
        Denial::Stopped(Err(Control::TimedOut))
    );
    for (cause, denial, ancestor) in [
        (Cause::WorkExhausted, Resource::WorkExhausted, None),
        (Cause::WorkLimitExceedsParent, Resource::WorkLimit, None),
        (Cause::WorkerLimitExceedsParent, Resource::WorkerLimit, None),
        (
            Cause::MemoryLimitExceedsParent,
            Resource::PolicyMemoryLimit,
            None,
        ),
        (
            Cause::PolicyMemoryExhausted {
                requested: 2,
                admitted: 1,
                ancestor: 0,
            },
            Resource::MemoryLimit {
                requested: 2,
                admitted: 1,
                level: Level::Policy,
            },
            Some(0),
        ),
        (
            Cause::ProcessMemoryExhausted {
                requested: 2,
                admitted: 1,
            },
            Resource::MemoryLimit {
                requested: 2,
                admitted: 1,
                level: Level::Process,
            },
            None,
        ),
        (
            Cause::DeclaredMemoryExhausted {
                requested: 2,
                admitted: 1,
            },
            Resource::MemoryLimit {
                requested: 2,
                admitted: 1,
                level: Level::Declared,
            },
            None,
        ),
        (
            Cause::WorkCounterOverflow,
            Resource::WorkCounterOverflow,
            None,
        ),
        (
            Cause::ResultCapacityExceeded,
            Resource::ResultCapacityExceeded,
            None,
        ),
        (
            Cause::ScratchCapacityExceeded,
            Resource::ScratchCapacityExceeded,
            None,
        ),
    ] {
        assert_eq!(
            grant_lookup_denial_kind(Kind::Execution(cause)),
            Denial::Stopped(Ok(Session::ExecutionResource {
                denial,
                partition_identity: None,
                policy_ancestor: ancestor
            }))
        );
    }
    assert_eq!(
        grant_lookup_denial_kind(Kind::CorruptIndexEntries),
        Denial::Refused(AuthorizationRefusal::RelationalObservationRejected)
    );
}
