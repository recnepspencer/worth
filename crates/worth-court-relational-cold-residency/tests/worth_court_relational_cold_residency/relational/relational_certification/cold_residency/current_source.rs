//! An actual published graph reopened on its exact checkpoint and journal prefix.
use super::{current_publications::*, selected_directory};
use std::{collections::BTreeSet, sync::atomic::Ordering, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::*, identity::*, runtime::RelationalRuntime};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    cold_recovery::{capture_source, update_entity},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::durable_runtime,
};

pub(super) struct CurrentFixture {
    pub runtime: RelationalRuntime,
    pub medium: Arc<TestDurableMedium>,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub entity: EntityId,
    pub target: EntityId,
    pub relation: RelationId,
    pub lineage: LineageId,
    pub heads: BTreeSet<RelationalDurableMemberLocator>,
    pub symbols: BTreeSet<RelationalDurableMemberLocator>,
    pub history: BTreeSet<RelationalDurableMemberLocator>,
}
impl CurrentFixture {
    pub fn reopened(request: &ExecutionRequest<'_, '_>) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let entity = create_entity(request, &producer, "head-only-before");
        let target = create_entity(request, &producer, "head-only-target");
        let committed =
            create_relation_outcome(request, &producer, entity, target, "head-only-edge");
        let relation = changed_relations(&committed)[0];
        release_test_commit_snapshot(&producer, &committed);
        update_entity(request, &producer, entity, "head-only-current");
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        let lineage = producer
            .lineage_node_at_observation(request, &basis.observation(), entity)
            .unwrap()
            .unwrap()
            .node()
            .lineage_id();
        drop(basis);
        let (runtime, source, directory, receiver_medium) =
            capture_and_reopen(request, producer, medium, entity);
        let regions = || {
            directory
                .mirror
                .iter()
                .chain(directory.roots.iter().flat_map(|root| &root.regions))
                .filter(|region| region.partition_id == entity.partition_id)
        };
        let history = regions().map(|region| region.history).collect();
        let heads = regions().map(|region| region.head).collect();
        let symbols = directory.symbols.iter().map(|page| page.member).collect();
        Self {
            runtime,
            medium: receiver_medium,
            source,
            entity,
            target,
            relation,
            lineage,
            heads,
            symbols,
            history,
        }
    }
}

pub(super) fn capture_and_reopen(
    request: &ExecutionRequest<'_, '_>,
    producer: RelationalRuntime,
    medium: Arc<TestDurableMedium>,
    entity: EntityId,
) -> (
    RelationalRuntime,
    Arc<ImmutableCheckpointMemberSource>,
    selected_directory::Directory,
    Arc<TestDurableMedium>,
) {
    let original = capture_source(request, producer, medium, entity, false);
    let directory = selected_directory::read(&original.input);
    let receiver_medium = TestDurableMedium::new();
    let attachment = receiver_medium.attachment();
    for artifact in original.medium.committed_artifacts().unwrap() {
        assert!(matches!(
            attachment.publish(&artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable_runtime(&receiver_medium);
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
    assert_eq!(original.source.reads.load(Ordering::Relaxed), 0);
    (runtime, original.source, directory, receiver_medium)
}
