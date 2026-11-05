use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenial as Denial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind as Kind;
use worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel as Level;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalReentryOutcome as Outcome;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::application_operation_reentry::sealed_commit::denial::classify_denial;
pub fn resources() -> Vec<Resource> {
    let mut causes = vec![
        Resource::WorkExhausted,
        Resource::WorkCounterOverflow,
        Resource::ResultCapacityExceeded,
        Resource::ScratchCapacityExceeded,
        Resource::CapacityOverflow,
        Resource::ChargedBytesOverflow,
        Resource::NestedLeaseMisuse,
        Resource::NoActiveExecutionScope,
    ];
    for level in [Level::Policy, Level::Process, Level::Declared] {
        causes.push(Resource::MemoryLimit {
            requested: 31,
            admitted: 17,
            level,
        });
    }
    causes
}
pub fn execution_cases() -> Vec<(Session, Kind)> {
    let mut cases = vec![
        (
            Session::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            Kind::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
        ),
        (
            Session::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
            Kind::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
        ),
        (
            Session::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
            Kind::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
        ),
        (
            Session::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
            Kind::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
        ),
    ];
    cases.extend(resources().into_iter().map(|denial| {
        (
            Session::ExecutionResource {
                denial,
                partition_identity: Some(7),
                policy_ancestor: None,
            },
            Kind::ExecutionResource {
                denial,
                partition_identity: Some(7),
                policy_ancestor: None,
            },
        )
    }));
    cases
}
pub fn assert_preparation_retry(denial: &Denial) {
    match classify_denial(denial) {
        Outcome::RetryableCommitFailure(kind) => assert_eq!(kind, denial.kind()),
        Outcome::RetryableExecutionControlStopped(kind) => {
            assert_eq!(denial.execution_denial_cause(), Some(Err(kind)))
        }
        _ => panic!("HEAD-reachable preparation refusal changed retry posture: {denial:?}"),
    }
}
