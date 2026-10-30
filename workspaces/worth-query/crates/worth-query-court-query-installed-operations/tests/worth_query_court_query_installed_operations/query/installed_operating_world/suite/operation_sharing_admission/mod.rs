pub(crate) use worth_query_testkit_query::facade::query::suite::operation_sharing_admission::*;

#[test]
fn unsupported_and_foreign_candidates_stop_before_owner_registration() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut unsupported = configured_runtime()
        .workspace(request_execution, "projection-sharing-unsupported")
        .unwrap();
    let live = promote(
        request_execution,
        settle(&mut unsupported),
        &mut unsupported,
    );
    let resource_name = live.resource_name().to_string();
    let candidate = settle(&mut unsupported).into_lifecycle();
    let stop = match live.share_with(candidate, &mut unsupported) {
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => stop,
        domain::WorthQueryProjectionSharingOutcome::Shared(_) => {
            panic!("unsupported sharing was admitted")
        }
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionSharingDenialKind::ConsumerSupport
    );
    assert_eq!(stop.counters().support_posture_checks, 2);
    assert_eq!(stop.counters().owner_registrations, 0);
    unsupported
        .resolve_live_artifact_target(&resource_name)
        .unwrap();

    let mut owner = sharing_workspace(request_execution, "projection-sharing-foreign-owner");
    let mut foreign = sharing_workspace(request_execution, "projection-sharing-foreign-candidate");
    let live = promote(request_execution, settle(&mut owner), &mut owner);
    let resource_name = live.resource_name().to_string();
    let candidate = settle(&mut foreign).into_lifecycle();
    let stop = match live.share_with(candidate, &mut owner) {
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => stop,
        domain::WorthQueryProjectionSharingOutcome::Shared(_) => {
            panic!("foreign candidate was admitted")
        }
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionSharingDenialKind::CandidateNotCurrent
    );
    assert_eq!(stop.counters().owner_registrations, 0);
    assert_eq!(stop.counters().lease_issues, 0);
    owner.resolve_live_artifact_target(&resource_name).unwrap();
}

#[test]
fn singleton_support_denial_is_typed_and_stale_authority_takes_precedence() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut unsupported = configured_runtime()
        .workspace(request_execution, "projection-singleton-unsupported")
        .unwrap();
    let live = promote(
        request_execution,
        settle(&mut unsupported),
        &mut unsupported,
    );
    let stop = match live.into_managed_lease(&mut unsupported) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => stop,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(_) => {
            panic!("unsupported singleton sharing was admitted")
        }
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionLeaseAdmissionDenialKind::ConsumerSupport
    );
    assert_eq!(stop.counters().support_checks, 1);
    assert_eq!(stop.counters().owner_authority_checks, 0);
    assert_eq!(stop.counters().owner_registration_attempts, 0);
    assert_eq!(stop.counters().lease_issues, 0);
    assert_eq!(stop.counters().unrelated_route_scans, 0);

    let mut controlled = configured_runtime()
        .controlled_workspace(request_execution, "projection-sharing-stale-unsupported")
        .unwrap();
    let live = promote(request_execution, settle(&mut controlled), &mut controlled);
    let candidate = settle(&mut controlled).into_lifecycle();
    controlled.advance_domain_installation_generation().unwrap();
    let stop = match live.share_with(candidate, &mut controlled) {
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => stop,
        domain::WorthQueryProjectionSharingOutcome::Shared(_) => {
            panic!("stale unsupported pair was admitted")
        }
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionSharingDenialKind::CandidateNotCurrent
    );
    assert_eq!(stop.counters().support_posture_checks, 0);
}

#[test]
fn independently_authored_pair_order_converges_without_a_secondary_sharing_index() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let forward = admit_pair(request_execution, "projection-sharing-order-forward", false);
    let reverse = admit_pair(request_execution, "projection-sharing-order-reverse", true);
    assert_eq!(forward, reverse);
    assert_eq!(forward.owner_registrations, 1);
    assert_eq!(forward.lease_issues, 2);
    assert_eq!(forward.closure_comparisons, 1);
    assert_eq!(forward.closure_readmissions, 1);
    assert_eq!(
        forward.dependency_edges_compared,
        forward.dependency_edges_readmitted
    );
}
