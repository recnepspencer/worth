use worth_query::facade::{domain, foundation, read, runtime};

use super::installed_operation_fixture::{
    configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

pub(super) type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

#[test]
fn live_owner_and_current_candidate_become_two_move_only_leases() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("projection-sharing-live-current")
        .unwrap();
    let live = match settle(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("subject projection did not promote"),
    };
    let resource_name = live.resource_name().to_string();
    let candidate = settle(&mut workspace).into_lifecycle();

    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("sharing stopped at {:?}: {}", stop.kind(), stop.detail())
        }
    };
    assert_eq!(shared.counters().owner_registrations, 1);
    assert_eq!(shared.counters().lease_issues, 2);
    assert_eq!(shared.counters().unrelated_registry_scans, 0);
    let (subject, candidate) = shared.into_leases();
    assert_eq!(subject.owner_identity(), candidate.owner_identity());
    assert_ne!(subject.lease_identity(), candidate.lease_identity());
    assert!(subject.owner_identity().generation() > 0);
    assert!(subject.lease_identity().generation() > 0);

    let first = match subject.dispose(&mut workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("first lease disposal stopped: {}", stop.error())
        }
    };
    assert!(!first.release().owner_terminal());
    assert_eq!(first.release().counters().capability_checks, 1);
    assert_eq!(first.release().counters().owner_index_lookups, 1);
    assert_eq!(first.release().counters().lease_index_lookups, 1);
    assert_eq!(first.release().counters().lease_removals, 1);
    assert_eq!(first.release().counters().close_attempts, 0);
    assert_eq!(first.release().counters().unrelated_owner_scans, 0);
    assert_eq!(first.release().counters().unrelated_lease_scans, 0);
    workspace
        .resolve_live_artifact_target(&resource_name)
        .unwrap();
    let last = match candidate.dispose(&mut workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("last lease disposal stopped: {}", stop.error())
        }
    };
    assert!(last.release().owner_terminal());
    assert!(last.release().closeout_identity().is_some());
    assert_eq!(last.release().counters().owner_removals, 1);
    assert_eq!(last.release().counters().close_attempts, 1);
    assert_eq!(last.release().counters().close_completions, 1);
    assert!(workspace
        .resolve_live_artifact_target(&resource_name)
        .is_err());
}

#[test]
fn live_owner_can_enter_the_same_managed_lease_lifecycle_without_a_candidate() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("projection-sharing-singleton")
        .unwrap();
    let live = match settle(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("singleton projection did not promote"),
    };
    let resource_name = live.resource_name().to_string();
    let lease = match live.into_managed_lease(&mut workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("singleton admission stopped: {}", stop.detail())
        }
    };
    assert_eq!(
        lease.owner_identity().runtime_authority(),
        lease.lease_identity().runtime_authority()
    );
    let disposed = match lease.dispose(&mut workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("singleton disposal stopped: {}", stop.error())
        }
    };
    assert!(disposed.release().owner_terminal());
    assert!(workspace
        .resolve_live_artifact_target(&resource_name)
        .is_err());
}

#[test]
fn one_owner_drain_and_one_maintenance_receipt_fan_out_to_both_exact_leases() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("projection-sharing-fanout")
        .unwrap();
    let live = match settle(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("fanout subject did not promote"),
    };
    let candidate = settle(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("fanout sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();

    let empty_subject = subject.drain(&mut workspace).unwrap();
    let empty_candidate = candidate.drain(&mut workspace).unwrap();
    assert_eq!(empty_subject.maintenance_ordinal(), 1);
    assert_eq!(empty_candidate.maintenance_ordinal(), 1);
    assert!(empty_subject.delivery().is_empty());
    assert_eq!(empty_subject.counters().owner_drain_calls, 1);
    assert_eq!(empty_subject.counters().underlying_maintenance_passes, 0);
    assert_eq!(empty_subject.counters().fanout_targets, 2);
    assert_eq!(empty_subject.counters().this_lease_semantic_delivery, 0);

    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "shared-live-update")
        })
        .unwrap();
    let changed_subject = subject.drain(&mut workspace).unwrap();
    let changed_candidate = candidate.drain(&mut workspace).unwrap();
    assert_eq!(changed_subject.maintenance_ordinal(), 2);
    assert_eq!(changed_candidate.maintenance_ordinal(), 2);
    assert_eq!(
        changed_subject.owner_identity(),
        changed_candidate.owner_identity()
    );
    assert_ne!(
        changed_subject.lease_identity(),
        changed_candidate.lease_identity()
    );
    assert_eq!(changed_subject.counters().owner_drain_calls, 1);
    assert_eq!(changed_subject.counters().underlying_maintenance_passes, 1);
    assert_eq!(changed_subject.counters().lease_index_visits, 2);
    assert_eq!(changed_subject.counters().fanout_targets, 2);
    assert_eq!(changed_subject.counters().this_lease_view, 1);
    assert_eq!(changed_subject.counters().this_lease_semantic_delivery, 1);
    assert_eq!(changed_subject.counters().unrelated_owner_scans, 0);
    assert_eq!(changed_subject.counters().unrelated_lease_scans, 0);
    assert_eq!(changed_subject.delivery(), changed_candidate.delivery());
    assert_eq!(
        changed_subject.impact().class(),
        changed_candidate.impact().class()
    );
    assert!(std::ptr::eq(
        changed_subject.impact(),
        changed_candidate.impact()
    ));
}

#[test]
fn foreign_workspace_cannot_release_a_runtime_affine_lease() {
    let mut owner = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("projection-sharing-owner")
        .unwrap();
    let mut foreign = configured_runtime()
        .workspace("projection-sharing-foreign")
        .unwrap();
    let live = match settle(&mut owner).into_lifecycle().promote(&mut owner) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("foreign-release subject did not promote"),
    };
    let resource_name = live.resource_name().to_string();
    let lease = match live.into_managed_lease(&mut owner) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("singleton admission stopped: {}", stop.detail())
        }
    };
    let drain_stop = match lease.drain(&mut foreign) {
        Err(stop) => stop,
        Ok(_) => panic!("foreign workspace drained an owner-affine lease"),
    };
    assert_eq!(drain_stop.counters().workspace_capability_checks, 1);
    assert_eq!(drain_stop.counters().abandoned_owner_index_lookups, 0);
    assert_eq!(drain_stop.counters().runtime_affinity_checks, 0);
    assert_eq!(drain_stop.counters().owner_maintenance_drains, 0);
    let lease = match lease.dispose(&mut foreign) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            assert_eq!(stop.counters().capability_checks, 1);
            assert_eq!(stop.counters().owner_index_lookups, 0);
            assert_eq!(stop.counters().lease_index_lookups, 0);
            assert_eq!(stop.counters().close_attempts, 0);
            stop.into_lease()
        }
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(_) => {
            panic!("foreign workspace released an owner-affine lease")
        }
    };
    owner.resolve_live_artifact_target(&resource_name).unwrap();
    match lease.dispose(&mut owner) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => {
            assert!(disposed.release().owner_terminal())
        }
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("owner retry failed: {}", stop.error())
        }
    }
}

#[test]
fn dropped_peer_is_reaped_from_a_pending_epoch_and_last_drop_closes_once() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("projection-sharing-drop-reap")
        .unwrap();
    let live = match settle(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("drop-reap subject did not promote"),
    };
    let resource_name = live.resource_name().to_string();
    let candidate = settle(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("drop-reap sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    assert_eq!(
        subject.drain(&mut workspace).unwrap().maintenance_ordinal(),
        1
    );
    drop(candidate);
    let after_peer_drop = subject.drain(&mut workspace).unwrap();
    assert_eq!(after_peer_drop.maintenance_ordinal(), 2);
    assert_eq!(
        after_peer_drop
            .drain_counters()
            .abandoned_owner_index_lookups,
        1
    );
    assert_eq!(after_peer_drop.drain_counters().abandoned_leases_reaped, 1);
    assert_eq!(after_peer_drop.drain_counters().unrelated_owner_scans, 0);
    workspace
        .resolve_live_artifact_target(&resource_name)
        .unwrap();
    drop(subject);
    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "reap-trigger")
        })
        .unwrap();
    assert!(workspace
        .resolve_live_artifact_target(&resource_name)
        .is_err());
}

#[test]
fn failed_last_close_retains_the_exact_lease_and_backend_until_retry() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .fail_next_live_closes(1)
        .workspace("projection-sharing-close-retry")
        .unwrap();
    let live = match settle(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("close-retry subject did not promote"),
    };
    let resource_name = live.resource_name().to_string();
    let lease = match live.into_managed_lease(&mut workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("singleton admission stopped: {}", stop.detail())
        }
    };
    let lease = match lease.dispose(&mut workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            assert_eq!(stop.counters().capability_checks, 1);
            assert_eq!(stop.counters().owner_index_lookups, 1);
            assert_eq!(stop.counters().lease_index_lookups, 1);
            assert_eq!(stop.counters().owner_removals, 1);
            assert_eq!(stop.counters().close_attempts, 1);
            assert_eq!(stop.counters().close_completions, 0);
            assert_eq!(stop.counters().owner_reinsertions, 1);
            stop.into_lease()
        }
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(_) => {
            panic!("injected backend close failure was not retained")
        }
    };
    workspace
        .resolve_live_artifact_target(&resource_name)
        .unwrap();
    match lease.dispose(&mut workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => {
            assert!(disposed.release().owner_terminal());
            assert!(disposed.release().closeout_identity().is_some());
        }
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("last-close retry failed: {}", stop.error())
        }
    }
    assert!(workspace
        .resolve_live_artifact_target(&resource_name)
        .is_err());
}

pub(super) fn settle(workspace: &mut runtime::WorthQueryWorkspace) -> SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
}
