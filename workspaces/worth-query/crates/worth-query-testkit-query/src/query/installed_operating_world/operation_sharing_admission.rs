pub use worth_query::facade::{domain, runtime};

pub use super::installed_operation_fixture::configured_runtime;
pub use super::operation_sharing::settle;

pub fn admit_pair(
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

pub fn singleton(
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

pub fn promote(
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

pub fn sharing_workspace(
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
