use crate::workflow_request::workflow_request;
use worth_query::facade::runtime::ExecutionRequest;
use worth_query::facade::{certification, domain, runtime};

use super::installed_operation_fixture::{
    consume_empty_invalidation_epoch as consume_empty_epoch, lineage_invalidation_workspace,
    settle_native, InvalidationLease as Lease,
};
use super::operation_lineage::{bind, intent};

#[test]
fn cert_reexecution_and_live_maintenance_converge_without_replay_authority_promotion() {
    let mut workspace = lineage_invalidation_workspace("invalidation-replay-convergence").unwrap();
    let live = match settle_native(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("lineage projection did not promote"),
    };
    let candidate = settle_native(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("lineage projection did not share: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    consume_empty_epoch(&mut workspace, &subject, &candidate);

    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            intent(),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let original_delivery = subject.drain(&mut workspace).unwrap();
    let _original_peer = candidate.drain(&mut workspace).unwrap();
    let original_impact = original_delivery.impact().semantic_projection();
    let original_observation = observe_delivery(&subject, original_delivery, &workspace);

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    let replay_delivery = subject.drain(&mut workspace).unwrap();
    let _replay_peer = candidate.drain(&mut workspace).unwrap();
    let replay_observation = observe_delivery(&subject, replay_delivery, &workspace);

    assert_eq!(&original_impact, original_observation.semantic.impact());
    assert!(original_observation
        .semantic
        .semantically_converges_with(&replay_observation.semantic));
    assert_eq!(
        original_observation.semantic.canonical_bytes(),
        replay_observation.semantic.canonical_bytes()
    );
    assert_eq!(
        original_observation.admitted_bytes,
        replay_observation.admitted_bytes
    );
    assert_eq!(
        original_observation.foundational,
        replay_observation.foundational
    );
    assert_eq!(
        original_observation.compatibility,
        domain::WorthQueryInvalidationCompatibilityOutcome::SharedEquivalentContinuity
    );
    assert_eq!(
        original_observation.compatibility,
        replay_observation.compatibility
    );
    assert_eq!(
        original_observation.maintenance_ordinal + 1,
        replay_observation.maintenance_ordinal
    );
    assert_eq!(original_observation.counters, replay_observation.counters);
    assert_eq!(
        original_observation.epoch_counters,
        replay_observation.epoch_counters
    );
}

struct ObservedInvalidation {
    semantic: domain::WorthQueryConsumerInvalidationSemanticProjection,
    admitted_bytes: [u8; 32],
    compatibility: domain::WorthQueryInvalidationCompatibilityOutcome,
    foundational: domain::WorthQueryFoundationalInvalidationProjection,
    maintenance_ordinal: u64,
    counters: domain::WorthQueryConsumerInvalidationCounters,
    epoch_counters: domain::WorthQueryConsumerInvalidationEpochCounters,
}

fn observe_delivery(
    lease: &Lease,
    delivery: domain::WorthQuerySharedProjectionDelivery,
    workspace: &runtime::WorthQueryWorkspace,
) -> ObservedInvalidation {
    let delta = lease.consumer_invalidation_delta(delivery).unwrap();
    let semantic = delta.semantic_projection();
    let foundational = delta.foundational_projection();
    let maintenance_ordinal = delta.maintenance_ordinal();
    let counters = delta.counters();
    let epoch_counters = delta.epoch_counters();
    let admitted = lease
        .admit_consumer_invalidation_delta(delta, workspace)
        .unwrap_or_else(|_| panic!("current replay-convergence delta did not readmit"));
    let admitted = admitted
        .admitted_semantic_projection(workspace)
        .expect("current admitted delta retains semantic continuity");
    ObservedInvalidation {
        semantic,
        admitted_bytes: *admitted.canonical_bytes(),
        compatibility: admitted.compatibility(),
        foundational,
        maintenance_ordinal,
        counters,
        epoch_counters,
    }
}
