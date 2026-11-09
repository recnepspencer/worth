//! Retained original fork root and distinct cold mirror, from a real checkpoint.
use super::selected_directory;
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, history::BranchId, identity::*, mvcc::RelationalBranchObservation,
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    cold_recovery::{capture_source, update_entity},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::{create_entity_outcome, durable_runtime},
};
pub(super) struct PinFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub medium: Arc<TestDurableMedium>,
    pub historical_version: VersionId,
    pub entity: EntityId,
    _source_observation: RelationalBranchObservation,
}
pub(super) fn reopened(request: &ExecutionRequest<'_, '_>) -> PinFixture {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let created = create_entity_outcome(request, &producer, "retained-pin-source");
    let entity = changed_entities(&created)[0];
    let historical_version = created.version_id;
    release_test_commit_snapshot(&producer, &created);
    let (_, fork_basis) = producer
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    producer
        .fork_branch(request, BranchId("pin-retained-source".into()), fork_basis)
        .unwrap();
    let (_, retained_basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let extra = create_entity_outcome(request, &producer, "member-before");
    let extra_entity = changed_entities(&extra)[0];
    release_test_commit_snapshot(&producer, &extra);
    for i in 0..2 {
        update_entity(
            request,
            &producer,
            extra_entity,
            &format!("member-update-{i}"),
        );
    }
    let original = capture_source(request, producer, medium, extra_entity, false);
    drop(retained_basis);
    let directory = selected_directory::read(&original.input);
    let mirror = directory
        .mirror
        .iter()
        .find(|r| r.partition_id == entity.partition_id)
        .unwrap()
        .history;
    // Both coordinates come from the producer's actual declared directory.
    let historical = directory
        .roots
        .first()
        .unwrap()
        .regions
        .iter()
        .find(|r| r.partition_id == entity.partition_id)
        .unwrap()
        .history;
    assert_ne!(
        historical, mirror,
        "historical admission cannot witness the distinct mirror history"
    );
    let medium = TestDurableMedium::new();
    let attachment = medium.attachment();
    for bytes in original.source.commits.values() {
        let artifact =
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            original.input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(
        original.source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no region member"
    );
    let identity = runtime
        .branch_identity(&BranchId("pin-retained-source".into()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    assert_eq!(observation.version_id(), historical_version);
    assert!(runtime
        .entity_record_at_observation(request, &observation, entity)
        .unwrap()
        .is_some());
    PinFixture {
        runtime,
        source: original.source,
        medium,
        historical_version,
        entity,
        _source_observation: observation,
    }
}
