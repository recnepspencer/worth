//! A Query admission stop keeps the caller's resource cause in Bank.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;
#[test]
fn query_admission_does_not_fold_a_request_stop_to_unavailable() {
    for denial in [
        Resource::WorkExhausted,
        Resource::MemoryLimit {
            level: worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Declared,
            requested: 7,
            admitted: 3,
        },
    ] {
        let cause = Session::ExecutionResource {
            denial,
            partition_identity: None,
            policy_ancestor: None,
        };
        assert_eq!(
            admission(Query::ExecutionRequest(cause)),
            BankApplicationQueryAdmissionDenialKind::ExecutionRequest(cause)
        );
    }
}

use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
