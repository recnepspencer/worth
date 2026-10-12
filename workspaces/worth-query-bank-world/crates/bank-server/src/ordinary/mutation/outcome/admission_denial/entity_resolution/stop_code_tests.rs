//! Bank's own stopped-resolution code is independent of HTTP's action class.
use super::{BankEntityResolutionDenial as Denial, BankEntityResolutionDenialKind as Kind};
use worth_query_host::facade::application_contribution::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Execution;
#[test]
fn entity_stop_bank_code_preserves_controls_and_execution_request() {
    for (kind, code) in [
        (Kind::Cancelled, "cancelled"),
        (Kind::DeadlineExceeded, "deadline-exceeded"),
    ] {
        let denial = Denial { kind };
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.code(), code);
    }
    for cause in [
        Execution::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: None,
            policy_ancestor: None,
        },
        Execution::ExecutionResource {
            denial: Resource::MemoryLimit {
                requested: 43,
                admitted: 17,
                level: Level::Declared,
            },
            partition_identity: None,
            policy_ancestor: None,
        },
        Execution::ExecutionResource {
            denial: Resource::MemoryLimit {
                requested: 43,
                admitted: 17,
                level: Level::Process,
            },
            partition_identity: None,
            policy_ancestor: None,
        },
        Execution::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 2,
        },
        Execution::ExecutionResource {
            denial: Resource::NestedLeaseMisuse,
            partition_identity: None,
            policy_ancestor: None,
        },
    ] {
        let denial = Denial {
            kind: Kind::ExecutionRequest(cause),
        };
        assert_eq!(denial.kind(), Kind::ExecutionRequest(cause));
        assert_eq!(denial.code(), "execution-request");
    }
}
