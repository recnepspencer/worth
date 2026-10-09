use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationInterruption as Control,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::WorthQueryApplicationRequestMutationDenialKind as Mutation,
};
#[test]
fn opening_cause_projection_contract_is_equal_for_queries_and_mutations() {
    let resources = [
        (
            Resource::MemoryLimit {
                level: Level::Process,
                requested: 1,
                admitted: 0,
            },
            Kind::Unavailable,
            Next::Retry,
        ),
        (
            Resource::MemoryLimit {
                level: Level::Policy,
                requested: 1,
                admitted: 0,
            },
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::MemoryLimit {
                level: Level::Declared,
                requested: 1,
                admitted: 0,
            },
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::WorkExhausted,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::PolicyMemoryLimit,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::WorkLimit,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::RetainedBytesExhausted,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::ScratchCapacityExceeded,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::ResultCapacityExceeded,
            Kind::MalformedRequest,
            Next::CorrectRequest,
        ),
        (
            Resource::WorkerLimit,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::WorkCounterOverflow,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::CapacityOverflow,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::ChargedBytesOverflow,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::NestedLeaseMisuse,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::NoActiveExecutionScope,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Resource::EquivalenceContractUnavailable,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
    ];
    for (resource, kind, next) in resources {
        check(Denial::Resource(resource), kind, next);
    }
    for (denial, kind, next) in [
        (
            Denial::Interrupted(Control::Cancelled),
            Kind::Cancelled,
            Next::Retry,
        ),
        (
            Denial::Interrupted(Control::DeadlineExceeded),
            Kind::DeadlineExceeded,
            Next::Retry,
        ),
        (Denial::NestedStopped, Kind::InternalDenied, Next::None),
        (Denial::Panicked, Kind::InternalDenied, Next::None),
    ] {
        check(denial, kind, next);
    }
}
fn check(cause: Denial, kind: Kind, next: Next) {
    let expected = BankHttpDenial::new(kind, next);
    assert_eq!(
        super::super::mutation_application::request_mutation_denial(Mutation::ExecutionRequest(
            cause
        )),
        expected,
        "{cause:?}"
    );
    assert_eq!(
        super::super::query_denial::query_denial(
            bank_server::BankApplicationQueryDenial::ExecutionRequest(cause)
        ),
        expected,
        "{cause:?}"
    );
}

#[test]
fn custody_fault_has_one_projection_through_opening_and_provider_routes() {
    use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;
    for (opening, resource) in [
        (Denial::NestedOpening, Resource::NestedAdvancementOpening),
        (Denial::ForeignPhase, Resource::ForeignAdvancementPhase),
    ] {
        let direct = advancement(opening);
        let provider = super::super::mutation_application::pending_execution_denial(
            Session::ExecutionResource {
                denial: resource,
                partition_identity: None,
                policy_ancestor: None,
            },
        );
        assert_eq!(
            direct,
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
        );
        assert_eq!(provider, direct);
        check(opening, Kind::InternalDenied, Next::ContactOperator);
    }
}
