//! Literal descriptive refusals classify every sibling reader, without authority.
use super::super::{
    connection::ConnectionRefusal, identity::InspectionIdentityStop, report::InspectionReportDenial,
};
use super::*;
use worth_execution::{MapKernelFailure, ScopeStop};
use worth_foundational::facade::CanonicalBasisPreparationStop;
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;
use WorthQueryApplicationInspectionStage as Stage;
use WorthQueryApplicationInspectionStopKind as Kind;

fn denial(cause: InspectionAdmissionDenial) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(InspectionCause::Admission(cause), None)
}
fn captured<E>(cause: E) -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))
}
#[test]
fn sibling_resource_reader_borrows_the_exact_retained_native_cause() {
    use crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial as Read;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
    let absent = denial(InspectionAdmissionDenial::Read(Read::ForeignBasis));
    assert!(absent.resource_refusal().is_none());
    let request = denial(InspectionAdmissionDenial::Read(Read::RequestStop(
        Native::Cancelled,
    )));
    let InspectionCause::Admission(InspectionAdmissionDenial::Read(Read::RequestStop(expected))) =
        &request.cause
    else {
        panic!("literal original resource owner")
    };
    let Some(WorthQueryApplicationInspectionResourceRefusal::Native(actual)) =
        request.resource_refusal()
    else {
        panic!("actual native resource consumer")
    };
    assert!(
        std::ptr::eq(actual, expected),
        "borrow the actual retained owner"
    );
    assert_eq!(*actual, Native::Cancelled);
}
#[test]
fn sibling_kind_reader_preserves_named_admission_request_and_unavailable_cases() {
    let unavailable =
        WorthQueryApplicationInspectionDenial::new(InspectionCause::Unavailable, None);
    assert_eq!(unavailable.kind(), Kind::Unavailable);
    let admission = denial(InspectionAdmissionDenial::FacetDenied);
    assert_eq!(admission.kind(), Kind::Admission);
    let request = denial(InspectionAdmissionDenial::Request(
        worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled),
    ));
    assert_eq!(request.kind(), Kind::Request);
}
#[test]
fn sibling_canonical_reader_keeps_identity_and_bridge_stages() {
    let identity = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::Identity(InspectionIdentityStop::Basis(
            CanonicalBasisPreparationStop::AccountingOverflow,
        )),
    )));
    let bridge = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::Bridge(BridgeQueryObservationReportingStop::Basis(
            CanonicalBasisPreparationStop::AccountingOverflow,
        )),
    )));
    assert!(matches!(
        identity.canonical_refusal(),
        Some(
            WorthQueryApplicationInspectionCanonicalRefusal::AccountingOverflow(
                Stage::ReportIdentity
            )
        )
    ));
    assert!(matches!(
        bridge.canonical_refusal(),
        Some(
            WorthQueryApplicationInspectionCanonicalRefusal::AccountingOverflow(
                Stage::ReportBridge
            )
        )
    ));
    let foreign = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::ForeignComparison,
    )));
    assert!(foreign.canonical_refusal().is_none());
}
#[test]
fn sibling_connection_reader_does_not_absorb_another_admission_stage() {
    use super::super::WorthQueryProducerConnectionDenial as Connection;
    let foreign = denial(InspectionAdmissionDenial::Connection(captured(
        ConnectionRefusal::General(Connection::ForeignSource),
    )));
    assert_eq!(
        foreign.producer_connection_refusal(),
        Some(Connection::ForeignSource)
    );
    let facet = denial(InspectionAdmissionDenial::FacetDenied);
    assert!(facet.producer_connection_refusal().is_none());
}
#[test]
fn sibling_evaluation_reader_keeps_exact_evaluation_and_excludes_other_connections() {
    use super::super::WorthQueryProducerConnectionDenial as Connection;
    use super::super::WorthQueryProducerEvaluationDenial as Evaluation;
    let evaluation = denial(InspectionAdmissionDenial::Connection(captured(
        ConnectionRefusal::General(Connection::Evaluation(
            Evaluation::OriginalCaptureUnavailable,
        )),
    )));
    assert_eq!(
        evaluation.producer_evaluation_refusal(),
        Some(Evaluation::OriginalCaptureUnavailable)
    );
    let foreign = denial(InspectionAdmissionDenial::Connection(captured(
        ConnectionRefusal::General(Connection::ForeignSource),
    )));
    assert!(foreign.producer_evaluation_refusal().is_none());
}
#[test]
fn sibling_materialization_reader_does_not_absorb_another_admission_stage() {
    use crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal as Materialization;
    let materialization = denial(InspectionAdmissionDenial::Rich(captured(
        Materialization::OriginalDisclosureIncomplete,
    )));
    assert_eq!(
        materialization.materialization_refusal(),
        Some(Materialization::OriginalDisclosureIncomplete)
    );
    let facet = denial(InspectionAdmissionDenial::FacetDenied);
    assert!(facet.materialization_refusal().is_none());
}
#[test]
fn sibling_execution_stop_reader_borrows_the_original_request_and_excludes_other_stages() {
    use worth_execution::{MapKernelStop as Stop, WorkCeilingDenial as Work};
    let request = denial(InspectionAdmissionDenial::Request(Work::Stopped(
        Stop::Cancelled,
    )));
    assert!(matches!(
        request.resource_refusal(),
        Some(WorthQueryApplicationInspectionResourceRefusal::Request(
            &Work::Stopped(Stop::Cancelled)
        ))
    ));
    let InspectionCause::Admission(InspectionAdmissionDenial::Request(expected)) = &request.cause
    else {
        panic!("actual request owner")
    };
    assert!(std::ptr::eq(
        request.execution_stop().expect("original request reader"),
        expected
    ));
    assert!(denial(InspectionAdmissionDenial::FacetDenied)
        .execution_stop()
        .is_none());
}
#[test]
fn sibling_interruption_reader_keeps_the_exact_original_control() {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
    let request = denial(InspectionAdmissionDenial::RequestControl(
        Interruption::Cancelled,
    ));
    assert_eq!(
        request.request_interruption(),
        Some(Interruption::Cancelled)
    );
    assert!(denial(InspectionAdmissionDenial::FacetDenied)
        .request_interruption()
        .is_none());
}

#[test]
fn sibling_canonical_digest_reader_keeps_identity_and_bridge_stages() {
    use worth_foundational::facade::CanonicalDigestAdmissionStop;
    let identity = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::Identity(InspectionIdentityStop::Digest(
            CanonicalDigestAdmissionStop::AccountingOverflow,
        )),
    )));
    let bridge = denial(InspectionAdmissionDenial::Report(captured(
        InspectionReportDenial::Bridge(BridgeQueryObservationReportingStop::Digest(
            CanonicalDigestAdmissionStop::AccountingOverflow,
        )),
    )));
    assert!(matches!(
        identity.canonical_refusal(),
        Some(
            WorthQueryApplicationInspectionCanonicalRefusal::AccountingOverflow(
                Stage::ReportIdentity
            )
        )
    ));
    assert!(matches!(
        bridge.canonical_refusal(),
        Some(
            WorthQueryApplicationInspectionCanonicalRefusal::AccountingOverflow(
                Stage::ReportBridge
            )
        )
    ));
}
