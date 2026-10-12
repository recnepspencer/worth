use super::*;

#[test]
fn binding_proof_exposes_linked_artifacts_and_witness_checks() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let progressed = progressed_route(&handle, "edge-a");
    let proof = handle.bind_route_from_target_proof(WorthQueryResolveRouteFromTargetRequest::new(
        WorthQueryRouteResolverSubject::Progression(progressed.clone()),
        RouteFamily::aspect_contract(),
    ));
    assert_eq!(proof.request().request_kind(), "resolve_route_from_target");
    assert_eq!(proof.witness_checks().len(), 1);
    assert!(proof.witness_checks()[0].did_pass());
    assert!(proof.resolved_target().is_some());
    assert_eq!(
        proof.linked_artifacts().progression_digest(),
        Some(progressed.progression_digest())
    );
}

#[test]
fn binding_digest_changes_when_required_aspect_contract_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let progressed = progressed_route(&handle, "edge-a");
    let exact = handle.bind_route_from_target_proof(WorthQueryResolveRouteFromTargetRequest::new(
        WorthQueryRouteResolverSubject::Progression(progressed.clone()),
        RouteFamily::aspect_contract(),
    ));
    let narrowed =
        handle.bind_route_from_target_proof(WorthQueryResolveRouteFromTargetRequest::new(
            WorthQueryRouteResolverSubject::Progression(progressed),
            WorthQueryDeclarationAspectContract::from_slices(
                &["selection.edge", "selection.material"],
                &[],
                &[],
                &[],
                &[],
            ),
        ));
    assert_ne!(exact.binding_digest(), narrowed.binding_digest());
}
