//! A mid-envelope comparison's failure (PR C, C4 r4): only an answer the
//! intermediate image cannot give is unknown, so qualified use keeps the
//! derivation for final support. Every other failure stops the edit.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::comparison_stops_hard_for_test as hard;
use worth_query_execution::facade::domain_computation::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Work,
};

#[test]
fn an_unavailable_answer_is_unknown() {
    assert!(!hard(&Denial::new(Kind::MissingAuthoritativeFact, "comparison")));
    for stop in [Stop::MembershipUnavailable, Stop::SummaryUnavailable, Stop::SourceStructureUnavailable] {
        assert!(!hard(&Denial::managed_index(Work::Controlled(stop), "comparison")));
    }
}

#[test]
fn cancellation_deadline_budgets_and_authority_stop() {
    for kind in [
        Kind::Cancelled,
        Kind::DeadlineExceeded,
        Kind::DecisionFactBudgetExceeded,
        Kind::CandidateCapacityExceeded,
        Kind::CurrentAuthorityDenied,
        Kind::StorageUnavailable,
        Kind::SourceChanged,
    ] {
        assert!(hard(&Denial::new(kind, "comparison")), "{kind:?}");
    }
}

#[test]
fn native_execution_storage_and_allocation_stop() {
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
    assert!(hard(&Denial::relational_read_stopped(Native::WorkLimitExceedsParent, "comparison")));
    let storage = worth_relational::facade::storage::RelationalStorageUnavailable {
        subject: worth_relational::facade::storage::RelationalStorageSubject::Region(
            worth_relational::facade::identity::PartitionId(7),
        ),
        cause: worth_relational::facade::storage::RelationalStorageUnavailableCause::SourceUnavailable,
    };
    assert!(hard(&Denial::native_storage_unavailable_for_test(storage, "comparison")));
    let owner = worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test();
    let lease = owner
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(0), Some(1)),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let cause = worth_execution::ExecutionByteBuffer::allocate(
        1,
        worth_execution::ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap_err();
    assert!(hard(&Denial::allocation_denied_for_test("comparison", cause)));
}

#[test]
fn a_managed_index_work_storage_or_capacity_stop_stops() {
    for stop in [
        Stop::Request(worth_execution::WorkCeilingDenial::Panicked),
        Stop::CapacityOverflow,
        Stop::NodeCapacityExceeded,
        Stop::SummaryMismatch,
        Stop::IncompleteCoverage,
    ] {
        assert!(hard(&Denial::managed_index(Work::Controlled(stop), "comparison")));
    }
}
