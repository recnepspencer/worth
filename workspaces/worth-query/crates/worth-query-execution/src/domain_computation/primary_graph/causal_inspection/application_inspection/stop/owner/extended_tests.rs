//! Literal retained causes exercise real state classification and public projection.
use super::*;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::{
    admission::InspectionPlanDenial, connection, report::InspectionReportDenial,
    target::InspectionTargetDenial,
};

fn captured<E>(cause: E) -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))
}
fn denial(cause: InspectionAdmissionDenial) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(InspectionCause::Admission(cause), None)
}

#[test]
fn inspection_preserves_principal_cause() {
    use crate::domain_computation::primary_graph::{
        WorthQueryPrincipalResolutionDenialKind as Kind,
        WorthQueryPrincipalResolutionRefusalKind as Refusal,
    };
    let original = WorthQueryPrincipalResolutionDenial::new(
        Kind::Refused(Refusal::UnknownPrincipal),
        "principal binding",
    );
    let denied = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Principal(original.clone()),
        None,
    );
    assert!(
        matches!(denied.owner_refusal(), Some(super::Refusal::Principal(actual)) if *actual == original)
    );
}

#[test]
fn inspection_preserves_graph_admission_cause() {
    use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial as Graph;
    let denied = denial(InspectionAdmissionDenial::Plan(captured(
        InspectionPlanDenial::Graph(Graph::InspectionAuthorityMismatch),
    )));
    assert!(matches!(
        denied.owner_refusal(),
        Some(Refusal::Graph(Graph::InspectionAuthorityMismatch))
    ));
}

#[test]
fn inspection_preserves_reporting_input_cause() {
    use worth_runtime_bridge::facade::BridgeQueryObservationReportingInputDenial as Input;
    let denied = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::Input(Input::MissingQueryObservation),
    )));
    assert!(matches!(
        denied.owner_refusal(),
        Some(Refusal::ReportingInput(Input::MissingQueryObservation))
    ));
}

#[test]
fn inspection_preserves_foreign_target_cause() {
    let denied = denial(InspectionAdmissionDenial::Target(
        InspectionTargetDenial::ForeignBinding,
    ));
    assert!(matches!(
        denied.owner_refusal(),
        Some(Refusal::ForeignTarget)
    ));
}

#[test]
fn inspection_preserves_session_start_cause() {
    let denied = denial(InspectionAdmissionDenial::Session(
        WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted,
    ));
    assert!(matches!(
        denied.owner_refusal(),
        Some(Refusal::Session(
            WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted
        ))
    ));
}

#[test]
fn inspection_preserves_original_capture_state_cause() {
    use crate::domain_computation::primary_graph::application_contribution::ProducerOutputEvaluationState;
    for original in [StoreDenial::ConflictingBody, StoreDenial::Representability] {
        let expected = OutputFieldCaptureFailure::Retention(original);
        let state = ProducerOutputEvaluationState::CaptureUnavailable(expected.clone());
        let connection = connection::require_compared(&state)
            .err()
            .expect("a failed capture cannot compare");
        let denied = denial(InspectionAdmissionDenial::Connection(captured(connection)));
        assert!(matches!(
            denied.producer_evaluation_refusal(),
            Some(
                super::super::super::WorthQueryProducerEvaluationDenial::OriginalCaptureUnavailable
            )
        ));
        let Some(Refusal::ProducerCapture(actual)) = denied.owner_refusal() else {
            panic!("capture cause must survive both consumers")
        };
        assert_eq!(actual, &expected);
        let Some(connection::ConnectionRefusal::Capture(retained)) = denied.connection_refusal()
        else {
            unreachable!()
        };
        assert!(std::ptr::eq(actual, retained));
    }
}

impl WorthQueryApplicationDisclosureContractDenial {
    pub(in crate::domain_computation) fn assert_inspection_contract_cause(self) {
        let expected = self.clone();
        let denied = denial(InspectionAdmissionDenial::DiagnosticContract(captured(crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Contract(self))));
        assert!(
            matches!(denied.owner_refusal(), Some(Refusal::DiagnosticContract(actual)) if *actual == expected)
        );
    }
}
