//! One row per cause pins the deliberate HTTP remedy change from HEAD.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Mutation,
    WorthQueryApplicationRequestMutationDenialKind as Request,
};
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;

#[test]
fn preparation_and_pending_execution_next_actions_are_explicit_per_cause() {
    use BankHttpNextAction::{ContactOperator as Operator, CorrectRequest as Correct, Retry};
    let resources = [
        ("WorkerLimitExceedsParent", Resource::WorkerLimit, Operator),
        (
            "MemoryLimitExceedsParent",
            Resource::PolicyMemoryLimit,
            Correct,
        ),
        ("WorkLimitExceedsParent", Resource::WorkLimit, Correct),
        (
            "PolicyMemoryExhausted",
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Policy,
            },
            Correct,
        ),
        (
            "ProcessMemoryExhausted",
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Process,
            },
            Retry,
        ),
        (
            "DeclaredMemoryExhausted",
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Declared,
            },
            Correct,
        ),
        (
            "ChargedBytesOverflow",
            Resource::ChargedBytesOverflow,
            Operator,
        ),
        (
            "UnrelatedNestedLease",
            Resource::NestedLeaseMisuse,
            Operator,
        ),
        (
            "NoActiveExecutionScope",
            Resource::NoActiveExecutionScope,
            Operator,
        ),
        (
            "EquivalenceContractUnavailable",
            Resource::EquivalenceContractUnavailable,
            Operator,
        ),
        (
            "WorkCounterOverflow",
            Resource::WorkCounterOverflow,
            Operator,
        ),
        ("WorkExhausted", Resource::WorkExhausted, Correct),
        (
            "ResultCapacityExceeded",
            Resource::ResultCapacityExceeded,
            Correct,
        ),
        (
            "ScratchCapacityExceeded",
            Resource::ScratchCapacityExceeded,
            Correct,
        ),
        ("MemoryOverflow", Resource::CapacityOverflow, Operator),
        (
            "RetainedBytesExhausted (managed only)",
            Resource::RetainedBytesExhausted,
            Correct,
        ),
    ];
    for (cause, resource, expected) in resources {
        let (_, preparation) =
            commit_denial(WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: resource,
                partition_identity: Some(7),
                policy_ancestor: None,
            });
        let pending = super::super::denial::pending_execution_denial(Session::ExecutionResource {
            denial: resource,
            partition_identity: Some(7),
            policy_ancestor: None,
        });
        assert_eq!(preparation.next_action, expected, "preparation {cause}");
        assert_eq!(pending.next_action, expected, "pending {cause}");
    }
    let faults = [
        (
            "NestedStopped",
            WorthQueryApplicationCommitDenialKind::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
            Session::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
            Retry,
        ),
        (
            "WorkerFailed",
            WorthQueryApplicationCommitDenialKind::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            Session::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            Operator,
        ),
        (
            "UncheckedCustomKernel",
            WorthQueryApplicationCommitDenialKind::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
            Session::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
            Operator,
        ),
        (
            "ExpectedIdentitiesNotCanonical",
            WorthQueryApplicationCommitDenialKind::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
            Session::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
            Operator,
        ),
        (
            "Busy",
            WorthQueryApplicationCommitDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 1,
            },
            Session::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 1,
            },
            Retry,
        ),
    ];
    for (cause, application, session, expected) in faults {
        assert_eq!(
            commit_denial(application).1.next_action,
            expected,
            "preparation {cause}"
        );
        assert_eq!(
            super::super::denial::pending_execution_denial(session).next_action,
            expected,
            "pending {cause}"
        );
    }
    // HTTP's wide metadata carrier is deferred. Preparation/pending controls
    // still use HEAD's ProviderRejected/Unavailable projection and keep Retry.
    for (cause, outcome) in [
        ("Cancelled", Mutation::Cancelled),
        ("DeadlineElapsed", Mutation::DeadlineExceeded),
    ] {
        assert_eq!(
            commit_denial(WorthQueryApplicationCommitDenialKind::ProviderRejected)
                .1
                .next_action,
            Retry,
            "preparation {cause}"
        );
        assert_eq!(
            super::super::denial::request_mutation_denial(Request::IdempotencyUnavailable)
                .next_action,
            Retry,
            "pending {cause}"
        );
        let observed = super::super::describe_outcome(cause.into(), Ok(outcome));
        let super::super::BankHttpMutationOutcome::NotApplied { denial, .. } = observed else {
            panic!("control stop unexpectedly applied");
        };
        assert_eq!(denial.next_action, Retry, "{cause}");
    }
}
