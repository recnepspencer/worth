use super::relational_execution_kind;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[test]
fn committed_storage_failure_keeps_its_subject_and_pre_effect_posture() {
    use worth_relational::facade::storage::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    let failure = RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(worth_relational::facade::identity::PartitionId(
            7,
        )),
        cause: RelationalStorageUnavailableCause::IntegrityMismatch,
    };
    let cause = Cause::StorageUnavailable(failure);
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    for attempt in [
        Attempt::native_storage_unavailable(failure, "transaction read"),
        Attempt::from(cause),
    ] {
        assert_eq!(attempt.storage_unavailable(), Some(&failure));
    }
    assert_eq!(
        relational_execution_kind(cause, None),
        Ok(Kind::StorageUnavailable(failure))
    );
    let crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(denial) =
        super::relational_execution_stop(cause, None)
    else {
        panic!("storage is a pre-effect refusal")
    };
    assert_eq!(denial.kind(), Kind::StorageUnavailable(failure));
    let read = worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(cause);
    assert_eq!(
        super::query_read_execution_kind(&read),
        Ok(Some(Kind::StorageUnavailable(failure)))
    );
}

#[test]
fn memory_causes_keep_their_capacity_and_owner_instead_of_becoming_work() {
    for (cause, expected_level, expected_ancestor) in [
        (
            Cause::PolicyMemoryExhausted {
                requested: 17,
                admitted: 11,
                ancestor: 2,
            },
            Level::Policy,
            Some(2),
        ),
        (
            Cause::ProcessMemoryExhausted {
                requested: 17,
                admitted: 11,
            },
            Level::Process,
            None,
        ),
        (
            Cause::DeclaredMemoryExhausted {
                requested: 17,
                admitted: 11,
            },
            Level::Declared,
            None,
        ),
    ] {
        let kind = relational_execution_kind(cause, Some(23)).unwrap();
        let Kind::ExecutionResource {
            denial:
                Resource::MemoryLimit {
                    requested,
                    admitted,
                    level,
                },
            partition_identity,
            policy_ancestor,
        } = kind
        else {
            panic!("memory must retain its typed cause: {kind:?}");
        };
        assert_eq!((requested, admitted), (17, 11));
        assert_eq!(level, expected_level);
        assert_eq!(partition_identity, Some(23));
        assert_eq!(policy_ancestor, expected_ancestor);
    }
    assert!(matches!(
        relational_execution_kind(Cause::WorkExhausted, None),
        Ok(Kind::ExecutionResource {
            denial: Resource::WorkExhausted,
            ..
        })
    ));
}

#[test]
fn native_allocator_refusal_is_neither_a_worker_panic_nor_a_policy_ceiling() {
    assert_eq!(
        relational_execution_kind(Cause::AllocationFailed, Some(23)),
        Ok(Kind::AllocationDenied)
    );
    assert_eq!(
        relational_execution_kind(Cause::WorkerFailed, Some(23)),
        Ok(Kind::ExecutionWorkerPanicked {
            partition_identity: Some(23)
        })
    );
}
