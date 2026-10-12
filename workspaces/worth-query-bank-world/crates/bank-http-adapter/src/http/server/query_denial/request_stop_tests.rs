//! HTTP projection of an already-raised query admission stop.
use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
    primary_graph::WorthQueryProviderSessionDenialKind as Session,
};
#[test]
fn query_admission_request_stops_keep_the_existing_request_instruction() {
    // The opening contract treats insufficient caller work or declared bytes
    // as a correctable request; current process pressure alone is retried.
    for denial in [
        Resource::WorkExhausted,
        Resource::MemoryLimit {
            level: WorthQueryMemoryLimitLevel::Declared,
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
            admission(BankApplicationQueryAdmissionDenialKind::ExecutionRequest(
                cause
            )),
            BankHttpDenial::new(Kind::MalformedRequest, Next::CorrectRequest)
        );
    }
}
