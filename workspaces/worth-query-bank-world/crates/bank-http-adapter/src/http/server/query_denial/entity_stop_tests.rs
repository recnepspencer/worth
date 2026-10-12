//! Entity resolution uses the shipped HTTP instruction and wire status/code.
use super::{entity, BankEntityResolutionDenialKind as Entity, Kind, Next};
use worth_query_host::facade::application_contribution::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Execution;
fn resource(denial: Resource) -> Entity {
    Entity::ExecutionRequest(Execution::ExecutionResource {
        denial,
        partition_identity: Some(3),
        policy_ancestor: Some(2),
    })
}
fn assert_answer(cause: Entity, kind: Kind, action: Next, status: u16, code: &str) {
    let answer = entity(cause);
    assert_eq!(
        answer,
        super::super::super::protocol::BankHttpDenial::new(kind, action)
    );
    assert_eq!(
        super::super::routes::response_status(answer.kind).as_u16(),
        status
    );
    assert_eq!(
        serde_json::to_value(answer).unwrap(),
        serde_json::json!({"kind":code, "next_action": serde_json::to_value(action).unwrap()})
    );
}
#[test]
fn entity_stop_controls_keep_retry_status_and_wire_code() {
    assert_answer(
        Entity::Cancelled,
        Kind::Cancelled,
        Next::Retry,
        408,
        "cancelled",
    );
    assert_answer(
        Entity::DeadlineExceeded,
        Kind::DeadlineExceeded,
        Next::Retry,
        408,
        "deadline_exceeded",
    );
}
#[test]
fn entity_stop_request_ceilings_ask_for_correction() {
    for denial in [
        Resource::WorkExhausted,
        Resource::MemoryLimit {
            requested: 43,
            admitted: 17,
            level: Level::Declared,
        },
        Resource::MemoryLimit {
            requested: 43,
            admitted: 17,
            level: Level::Policy,
        },
    ] {
        assert_answer(
            resource(denial),
            Kind::MalformedRequest,
            Next::CorrectRequest,
            400,
            "malformed_request",
        );
    }
}
#[test]
fn entity_stop_process_pressure_and_capacity_keep_retry() {
    for cause in [
        resource(Resource::MemoryLimit {
            requested: 43,
            admitted: 17,
            level: Level::Process,
        }),
        Entity::ExecutionRequest(Execution::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 2,
        }),
    ] {
        assert_answer(cause, Kind::Unavailable, Next::Retry, 503, "unavailable");
    }
}
#[test]
fn entity_stop_misuse_names_the_operator() {
    assert_answer(
        resource(Resource::NestedLeaseMisuse),
        Kind::InternalDenied,
        Next::ContactOperator,
        500,
        "internal_denied",
    );
}
