use super::{execution, BankApplicationOneShotDenialKind as Execution, Kind, Next};
use worth_query_host::facade::runtime::ExecutionPhysicalAllocationDenialKind as Physical;

#[test]
fn projection_inspection_denials_keep_the_existing_unavailable_http_contract() {
    for cause in [
        Execution::ProjectionInspectionUnavailable,
        Execution::ProjectionInspectionIncomplete,
    ] {
        let projected = execution(cause);
        assert_eq!(projected.kind, Kind::Unavailable);
        assert_eq!(projected.next_action, Next::Retry);
        assert_eq!(
            super::super::routes::response_status(projected.kind).as_u16(),
            503
        );
        assert_eq!(
            serde_json::to_value(projected).unwrap(),
            serde_json::json!({"kind": "unavailable", "next_action": "retry"})
        );
    }
}

#[test]
fn source_observation_allocation_keeps_physical_fault_recovery_in_http() {
    for (kind, expected_kind, expected_next, status, expected_json) in [
        (
            Physical::Allocator,
            Kind::Unavailable,
            Next::Retry,
            503,
            serde_json::json!({"kind": "unavailable", "next_action": "retry"}),
        ),
        (
            Physical::Layout,
            Kind::InternalDenied,
            Next::ContactOperator,
            500,
            serde_json::json!({"kind": "internal_denied", "next_action": "contact_operator"}),
        ),
        (
            Physical::CapacityMismatch,
            Kind::InternalDenied,
            Next::ContactOperator,
            500,
            serde_json::json!({"kind": "internal_denied", "next_action": "contact_operator"}),
        ),
        (
            Physical::WriteBeyondReserved,
            Kind::InternalDenied,
            Next::ContactOperator,
            500,
            serde_json::json!({"kind": "internal_denied", "next_action": "contact_operator"}),
        ),
        (
            Physical::IncompleteSeal,
            Kind::InternalDenied,
            Next::ContactOperator,
            500,
            serde_json::json!({"kind": "internal_denied", "next_action": "contact_operator"}),
        ),
    ] {
        let projected = execution(Execution::SourceObservationAllocation {
            kind,
            requested_payload_bytes: Some(4096),
        });
        assert_eq!(projected.kind, expected_kind);
        assert_eq!(projected.next_action, expected_next);
        assert_eq!(
            super::super::routes::response_status(projected.kind).as_u16(),
            status
        );
        assert_eq!(serde_json::to_value(projected).unwrap(), expected_json);
    }
}
