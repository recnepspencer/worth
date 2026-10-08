use super::*;
use crate::suite::installed_operation_fixture::{
    resource_admission_workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};
use installed::operation::{
    WorthQueryExecutionBoundary as Boundary, WorthQueryResourceDimension as Dimension,
    WorthQueryResourceLimitRequest as Limits, WorthQuerySemanticScaleAxis as Axis,
    WorthQuerySemanticScaleRequest as Scale,
};
use std::sync::atomic::Ordering;
use worth_proof::TransitionOutcome;
#[test]
fn external_atomic_resource_admission_preserves_present_support_and_capacity() {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::atomic(
        Scale::selective().with(Axis::CandidateItems, 3),
        Limits::selective().with(Dimension::RetainedBytes, 17),
        safe_point("atomic"),
    );
    let contract = contract([strategy("atomic", envelope.clone())]);
    let request = installed::operation::WorthQueryExecutionResourceRequest::atomic(
        envelope.scale_ceilings().clone(),
        envelope.resource_ceilings().clone(),
        safe_point("atomic"),
    )
    .unwrap();
    assert_eq!(request.boundary(), Boundary::Atomic);
    assert_eq!(
        envelope.bounded_step_contract(),
        Err("atomic-execution-is-not-bounded-step")
    );
    let capacity = std::sync::Arc::new(
        domain::WorthQueryFixedExecutionCapacity::new("external-atomic", 1).unwrap(),
    );
    let support = domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new(PROVIDER).unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new(ACCESS).unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new(ALLOCATOR).unwrap(),
        envelope.clone(),
        capacity.clone(),
    );
    let (mut workspace, contacts) =
        resource_admission_workspace("external-atomic", contract, support).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let first = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), request.clone(), &workspace)
        .unwrap();
    assert_eq!(first.resources().envelope(), &envelope);
    assert_eq!(capacity.active_attempts(), 1);
    let second = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), request.clone(), &workspace);
    let TransitionOutcome::Deferred(denial) = second else {
        panic!("Atomic must retain physical concurrency admission");
    };
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    drop(first);
    assert_eq!(capacity.active_attempts(), 0);
    let retry = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(ReadExecutionInput::default(), request, &workspace)
        .unwrap();
    let executed = retry.execute(&mut workspace).unwrap();
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(executed.resources().envelope(), &envelope);
    assert_eq!(capacity.active_attempts(), 0);
}
