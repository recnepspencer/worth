use worth_query::facade::domain;

use super::installed_operation_fixture::{
    configured_runtime, consume_empty_invalidation_epoch as consume_empty_epoch,
    materialized_invalidation_profile as materialized_profile, settle_native,
    settle_native_derived, shared_native_leases,
};

#[test]
fn foreign_runtime_rejects_an_other_runtime_delta() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut owner, owner_lease, owner_peer) =
        shared_native_leases(request_execution, "invalidation-runtime-owner");
    consume_empty_epoch(&mut owner, &owner_lease, &owner_peer);
    owner
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "owner")
        })
        .unwrap();
    let delta = owner_lease
        .consumer_invalidation_delta(owner_lease.drain(&mut owner).unwrap())
        .unwrap();
    let _ = owner_peer.drain(&mut owner).unwrap();

    let (mut foreign, foreign_lease, foreign_peer) =
        shared_native_leases(request_execution, "invalidation-runtime-foreign");
    consume_empty_epoch(&mut foreign, &foreign_lease, &foreign_peer);
    let stopped = match foreign_lease.admit_consumer_invalidation_delta(delta, &foreign) {
        Err(stopped) => stopped,
        Ok(_) => panic!("a foreign runtime admitted another runtime's delta"),
    };
    assert_eq!(
        stopped.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
    assert_eq!(stopped.counters().live_source_authority_checks, 1);
    assert_eq!(stopped.counters().delta_authority_readmission_checks, 1);
    assert_eq!(stopped.counters().epoch_readmission_checks, 0);
    assert_eq!(stopped.counters().sharing_readmission_checks, 0);
}

#[test]
fn different_declared_access_capability_rejects_the_delta() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(request_execution, "invalidation-declaration-drift")
        .unwrap();
    let id = match settle_native(&mut workspace)
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("id projection did not promote"),
    };
    let id = match id.into_managed_lease(&mut workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        _ => panic!("id lease did not admit"),
    };
    let derived = match settle_native_derived(&mut workspace)
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("derived projection did not promote"),
    };
    let derived = match derived.into_managed_lease(&mut workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        _ => panic!("derived lease did not admit"),
    };
    assert!(id.drain(&mut workspace).unwrap().delivery().is_empty());
    assert!(derived.drain(&mut workspace).unwrap().delivery().is_empty());
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "id-change")
        })
        .unwrap();
    let delta = id
        .consumer_invalidation_delta(id.drain(&mut workspace).unwrap())
        .unwrap();
    let stopped = match derived.admit_consumer_invalidation_delta(delta, &workspace) {
        Err(stopped) => stopped,
        Ok(_) => panic!("a differently declared access capability admitted the id delta"),
    };
    assert_eq!(
        stopped.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
}

#[test]
fn installation_drift_rejects_and_retains_the_exact_delta() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .controlled_workspace(request_execution, "invalidation-stale-generation")
        .unwrap();
    let live = match settle_native(&mut workspace)
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("stale-generation subject did not promote"),
    };
    let lease = match live.into_managed_lease(&mut workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("stale-generation lease stopped: {}", stop.detail())
        }
    };
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "before-drift")
        })
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let ordinal = delta.maintenance_ordinal();
    workspace.advance_domain_installation_generation().unwrap();
    let stop = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Err(stop) => stop,
        Ok(_) => panic!("stale installation admitted a prior-generation delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
    assert_eq!(stop.into_delta().maintenance_ordinal(), ordinal);
}

#[test]
fn superseded_epoch_rejects_old_authority_and_fresh_foundational_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, subject, candidate) =
        shared_native_leases(request_execution, "invalidation-stale-epoch");
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "epoch-one")
        })
        .unwrap();
    let old = subject
        .consumer_invalidation_delta(subject.drain(&mut workspace).unwrap())
        .unwrap();
    let old_ordinal = old.maintenance_ordinal();
    let peer_old = candidate
        .consumer_invalidation_delta(candidate.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted_peer = match candidate.admit_consumer_invalidation_delta(peer_old, &workspace) {
        Ok(admitted) => admitted,
        Err(_) => panic!("current peer epoch did not initially admit"),
    };

    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "epoch-two")
        })
        .unwrap();
    let current = subject.drain(&mut workspace).unwrap();
    let _peer = candidate.drain(&mut workspace).unwrap();
    assert!(current.maintenance_ordinal() > old_ordinal);

    let stopped = match subject.admit_consumer_invalidation_delta(old, &workspace) {
        Err(stopped) => stopped,
        Ok(_) => panic!("superseded maintenance epoch admitted an old delta"),
    };
    assert_eq!(
        stopped.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
    assert_eq!(stopped.into_delta().maintenance_ordinal(), old_ordinal);
    assert!(matches!(
        admitted_peer.materialize_foundational_projection(&workspace, materialized_profile()),
        Err(domain::WorthQueryFoundationalInvalidationMaterializationStop::ForeignOrStaleLease)
    ));
    let stopped = match admitted_peer.attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch,
        "stale-patch",
    ) {
        Err(stopped) => stopped,
        Ok(_) => panic!("superseded admitted delta attached a stale consequence"),
    };
    assert_eq!(
        stopped.kind(),
        domain::WorthQueryConsumerConsequenceAdmissionStopKind::ForeignOrStaleInvalidation
    );
    assert_eq!(stopped.into_consumer_authored(), "stale-patch");
}
