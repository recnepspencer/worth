use std::sync::atomic::Ordering;

use worth_proof::TransitionOutcome;
use worth_query::facade::installed;

use super::*;
use crate::suite::installed_operation_fixture::{
    resource_admission_workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

#[test]
fn concurrent_arrivals_saturate_and_drop_releases_capacity() {
    let capacity = envelope(
        2,
        2,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("live-pressure-chunk"),
    );
    let (workspace, provider_contacts) = resource_admission_workspace(
        "resource-live-saturation",
        contract([strategy("bounded", capacity.clone())]),
        support(capacity),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();

    let first = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), bounded_request(), &workspace)
        .unwrap();
    let second = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), bounded_request(), &workspace)
        .unwrap();

    assert_eq!(first.resources().counters().capacity_reservation_checks, 1);
    assert_eq!(first.resources().counters().capacity_reservations, 1);
    assert_eq!(second.resources().counters().capacity_reservations, 1);

    let third = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), bounded_request(), &workspace);
    let TransitionOutcome::Deferred(denial) = third else {
        panic!("the third concurrent arrival must encounter live saturation")
    };
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert_eq!(denial.counters().capacity_reservation_checks, 1);
    assert_eq!(denial.counters().capacity_reservations, 0);
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(provider_contacts.load(Ordering::SeqCst), 0);

    drop(first);
    let replacement = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), bounded_request(), &workspace)
        .unwrap();
    assert_eq!(replacement.resources().counters().capacity_reservations, 1);
    assert_eq!(provider_contacts.load(Ordering::SeqCst), 0);
}

fn bounded_request() -> installed::operation::WorthQueryExecutionResourceRequest {
    request(
        installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
        installed::operation::WorthQueryResourceLimitRequest::bounded(1),
        safe_point("live-pressure-chunk"),
    )
}
