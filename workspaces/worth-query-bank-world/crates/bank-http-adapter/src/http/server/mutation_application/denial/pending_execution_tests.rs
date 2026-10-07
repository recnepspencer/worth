//! Pending publication preserves the remedy through both HTTP entry paths.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Kind;

#[test]
fn mapping_contract_pending_execution_preserves_the_actor_that_can_act() {
    // HTTP receives an already typed refusal. Its contract selects a remedy;
    // the real leased owner refusal is exercised at the Query boundary.
    let cases = [
        (
            Kind::ExecutionResource {
                denial: Resource::MemoryLimit {
                    requested: 31,
                    admitted: 17,
                    level: Level::Process,
                },
                partition_identity: Some(7),
                policy_ancestor: None,
            },
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::Retry,
        ),
        (
            Kind::ExecutionResource {
                denial: Resource::WorkLimit,
                partition_identity: Some(7),
                policy_ancestor: None,
            },
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        (
            Kind::ExecutionResource {
                denial: Resource::WorkerLimit,
                partition_identity: Some(7),
                policy_ancestor: None,
            },
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        (
            Kind::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    ];
    for (kind, category, next) in cases {
        let expected = BankHttpDenial::new(category, next);
        assert_eq!(
            request_mutation_denial(
                WorthQueryApplicationRequestMutationDenialKind::IdempotencyExecutionDenied(kind)
            ),
            expected,
            "request path: {kind:?}"
        );
        assert_eq!(
            crate::http::server::estate_denial::estate_denial(
                bank_server::BankEstateProgressionDenial::Idempotency(
                    bank_server::BankEstateIdempotencyResolutionDenial::ExecutionDenied(kind)
                )
            ),
            expected,
            "estate path: {kind:?}"
        );
    }
}
