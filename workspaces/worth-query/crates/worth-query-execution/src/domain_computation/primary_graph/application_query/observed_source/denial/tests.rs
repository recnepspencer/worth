use super::WorthQuerySourceExpectationDenial;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;

#[test]
fn entry_custody_source_authorization_kind_is_unit_and_keeps_the_original_cause() {
    use crate::domain_computation::authorization::{
        WorthQueryOperationAuthorizationDenial as Authorization,
        WorthQueryOperationAuthorizationDenialKind as Kind,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    };
    let original = Authorization::new(Kind::Refused(Refusal::ExpiredAuthentication), "operation");
    let denial =
        WorthQuerySourceExpectationDenial::admission_authorization("source", original.clone());
    assert_eq!(
        denial.kind(),
        super::WorthQuerySourceExpectationDenialKind::AdmissionAuthorization
    );
    assert_eq!(denial.authorization_denial(), Some(&original));
    assert_eq!(
        denial.authorization_denial().unwrap().kind(),
        Kind::Refused(Refusal::ExpiredAuthentication)
    );
    assert_eq!(denial.retention_interruption(), None);
    assert_eq!(denial.execution_denial(), None);
}

#[test]
fn source_retention_observations_keep_original_caller_and_execution_stops() {
    for stop in [
        WorthQueryRequestInterruption::Cancelled,
        WorthQueryRequestInterruption::DeadlineExceeded,
    ] {
        let cause = WorthQuerySourceExpectationDenial::source_retention_denied(
            "observed source",
            StoreDenial::RequestInterruption(stop),
        );
        assert_eq!(cause.retention_interruption(), Some(stop));
        assert_eq!(cause.execution_denial(), None);
    }
    for stop in [MapKernelStop::Cancelled, MapKernelStop::DeadlineElapsed] {
        let original = WorkCeilingDenial::Stopped(stop);
        let cause = WorthQuerySourceExpectationDenial::source_retention_denied(
            "observed source",
            StoreDenial::ExecutionRequest(original),
        );
        assert_eq!(cause.execution_denial(), Some(&original));
        assert_eq!(cause.retention_interruption(), None);
    }
}
