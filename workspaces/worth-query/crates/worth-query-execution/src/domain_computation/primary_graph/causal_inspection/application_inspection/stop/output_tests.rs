//! Projection courts; these private causes are not producer receipts or join proof.
use super::super::{
    connection::ConnectionRefusal, WorthQueryProducerConnectionDenial as Connection,
    WorthQueryProducerOutputConnectionDenial as Output,
};
use super::*;
use worth_execution::{MapKernelFailure, ScopeStop};

fn connection(cause: ConnectionRefusal) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Admission(InspectionAdmissionDenial::Connection(CaptureStop::Kernel(
            ScopeStop::Failure(MapKernelFailure::Domain(cause)),
        ))),
        None,
    )
}

#[test]
fn output_detail_preserves_coarse_refusal_and_does_not_project_from_other_stops() {
    let refusal = connection(ConnectionRefusal::Output(
        Output::NativeRevisionNotCreatedByProducer,
    ));
    assert_eq!(
        refusal.producer_connection_refusal(),
        Some(Connection::OutputUnconnected)
    );
    assert_eq!(
        refusal.producer_output_refusal(),
        Some(Output::NativeRevisionNotCreatedByProducer)
    );
    assert!(refusal.resource_refusal().is_none());
    let foreign = connection(Connection::ForeignSource.into());
    assert_eq!(
        foreign.producer_connection_refusal(),
        Some(Connection::ForeignSource)
    );
    assert!(foreign.producer_output_refusal().is_none());
    let admission = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Admission(InspectionAdmissionDenial::ForeignIssuedAccess),
        None,
    );
    assert!(admission.producer_connection_refusal().is_none());
    assert!(admission.producer_output_refusal().is_none());
    let kernel = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Admission(InspectionAdmissionDenial::Connection(CaptureStop::Kernel(
            ScopeStop::Failure(MapKernelFailure::Panic),
        ))),
        None,
    );
    assert!(kernel.producer_connection_refusal().is_none());
    assert!(kernel.producer_output_refusal().is_none());
}
