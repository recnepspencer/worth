use worth_query::facade::{domain, runtime};

use super::installed_operation_fixture::configured_runtime;
use super::operation_sharing::settle;

#[test]
fn unsupported_and_foreign_candidates_stop_before_owner_registration() {
    let host_request = crate::test_execution_policy::serial_request();
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
    let host_request = crate::test_execution_policy::serial_request();
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
    let host_request = crate::test_execution_policy::serial_request();
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

#[test]
fn exact_owner_lookup_remains_bounded_with_unrelated_active_owners() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = sharing_workspace(request_execution, "projection-sharing-unrelated-owners");
    let unrelated_a = singleton(request_execution, &mut workspace);
    let unrelated_b = singleton(request_execution, &mut workspace);
    let live = promote(request_execution, settle(&mut workspace), &mut workspace);
    let candidate = settle(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("bounded sharing stopped: {}", stop.detail())
        }
    };
    assert_eq!(shared.counters().unrelated_registry_scans, 0);
    let (subject, candidate) = shared.into_leases();
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    assert_eq!(subject_delivery.counters().unrelated_owner_scans, 0);
    assert_eq!(subject_delivery.counters().unrelated_lease_scans, 0);
    assert_eq!(subject_delivery.counters().lease_index_visits, 2);
    assert_eq!(subject_delivery.drain_counters().owner_index_lookups, 1);
    assert_eq!(subject_delivery.drain_counters().lease_index_lookups, 1);
    assert_eq!(
        subject_delivery.drain_counters().sharing_readmission_checks,
        2
    );
    assert_eq!(subject_delivery.drain_counters().unrelated_owner_scans, 0);
    assert_eq!(subject_delivery.drain_counters().unrelated_lease_scans, 0);
    assert_eq!(
        subject_delivery.maintenance_ordinal(),
        candidate_delivery.maintenance_ordinal()
    );
    drop((unrelated_a, unrelated_b, subject, candidate));
}

fn admit_pair(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    reverse: bool,
) -> domain::WorthQueryProjectionSharingCounters {
    let mut workspace = sharing_workspace(request_execution, name);
    let first = settle(&mut workspace);
    let second = settle(&mut workspace);
    let (live_source, candidate_source) = if reverse {
        (second, first)
    } else {
        (first, second)
    };
    let live = promote(request_execution, live_source, &mut workspace);
    let shared = match live.share_with(candidate_source.into_lifecycle(), &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("ordered sharing stopped: {}", stop.detail())
        }
    };
    let counters = shared.counters();
    drop(shared);
    counters
}

fn singleton(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQuerySharedLiveProjectionLease<
    super::installed_operation_fixture::GeometryDomain,
    super::installed_operation_fixture::ReadVertex,
    super::installed_operation_fixture::ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    match promote(request_execution, settle(workspace), workspace).into_managed_lease(workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => {
            let counters = lease
                .singleton_admission_counters()
                .expect("singleton lease retains admission counters");
            assert_eq!(counters.support_checks, 1);
            assert_eq!(counters.owner_authority_checks, 1);
            assert_eq!(counters.owner_registration_attempts, 1);
            assert_eq!(counters.lease_issues, 1);
            assert_eq!(counters.unrelated_route_scans, 0);
            lease
        }
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("unrelated singleton stopped: {}", stop.detail())
        }
    }
}

fn promote(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    settled: super::operation_sharing::SettledProjection,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQueryLiveBoundDomainProjection<
    super::installed_operation_fixture::GeometryDomain,
    super::installed_operation_fixture::ReadVertex,
    super::installed_operation_fixture::ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    match settled
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("projection did not promote"),
    }
}

fn sharing_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> runtime::WorthQueryWorkspace {
    configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(request_execution, name)
        .unwrap()
}
