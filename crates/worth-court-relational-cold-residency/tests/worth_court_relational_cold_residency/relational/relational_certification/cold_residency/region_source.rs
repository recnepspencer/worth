//! Relation-bearing immutable regions recovered from their actual producer.
use super::{region_publications::*, selected_directory};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::*, identity::*, runtime::RelationalRuntime};
use worth_testkit_relational::{
    cold_recovery::capture_source,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::durable_runtime,
};
pub(super) struct ColdRoot {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub entities: Vec<EntityId>,
}
pub(super) fn recover_shape(
    request: &ExecutionRequest<'_, '_>,
    missing: Option<PartitionId>,
    count: u32,
    ring: bool,
) -> ColdRoot {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let mut entities = Vec::new();
    let mut targets = Vec::new();
    for id in 1..=count {
        entities.push(create_entity(request, &producer, "source", PartitionId(id)));
        targets.push(create_entity(request, &producer, "target", PartitionId(id)));
    }
    for (i, &entity) in entities.iter().enumerate() {
        create_relation(
            request,
            &producer,
            entity,
            if ring {
                entities[(i + 1) % entities.len()]
            } else {
                targets[i]
            },
        );
    }
    let original = capture_source(request, producer, medium, entities[0], false);
    let directory = selected_directory::read(&original.input);
    let source = if let Some(partition) = missing {
        let locator = directory.roots[0]
            .regions
            .iter()
            .find(|r| r.partition_id == partition)
            .unwrap()
            .history;
        original.source.without_member(locator)
    } else {
        original.source
    };
    let input = RelationalDurableRecoverySource::new(
        original.input.checkpoint().cloned(),
        original.input.tail().to_vec(),
        original.input.transitions().to_vec(),
        source.clone(),
    );
    let medium = TestDurableMedium::new();
    let attachment = medium.attachment();
    for bytes in source.commits.values() {
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
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    ColdRoot {
        runtime,
        source,
        entities,
    }
}
