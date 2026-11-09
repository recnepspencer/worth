//! A real bulk producer supplying an exact cold node page to concurrent readers.
use super::selected_directory::{self, Directory, Page};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, identity::*, lineage::LineageNode, runtime::RelationalRuntime,
    symbols::ClientKey, transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::{create_entity_outcome, durable_runtime, field_patch},
};

pub(super) struct NodeFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub expected: LineageNode,
    pub page: Page,
    pub directory: Directory,
}
pub(super) fn reopened(request: &ExecutionRequest<'_, '_>) -> NodeFixture {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let mut transaction = begin_main_transaction(&producer);
    transaction
        .push_batch(
            WorkerIntentBatch::new("paged-node-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..601)
                        .map(|id| ClientKey::raw(format!("node-{id:04}")))
                        .collect(),
                    field_patches: (0..601)
                        .map(|_| field_patch("name", "name", "node-payload"))
                        .collect(),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(request, &producer).unwrap();
    let entity = *changed_entities(&committed).last().unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let expected = {
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        producer
            .lineage_node_at_observation(request, &basis.observation(), entity)
            .unwrap()
            .unwrap()
            .node()
            .clone()
    };
    // The original fixture's ordinary fixture_from_runtime adds this exact commit.
    let created = create_entity_outcome(request, &producer, "member-before");
    let extra = changed_entities(&created)[0];
    release_test_commit_snapshot(&producer, &created);
    let ColdRecoverySourceFixture { input, source, .. } =
        capture_source(request, producer, medium, extra, false);
    source.damaged.store(false, Ordering::Relaxed);
    let directory = selected_directory::read(&input);
    assert!(
        directory.lineage.len() > 2,
        "the node population spans multiple real pages"
    );
    let page = *directory
        .lineage
        .iter()
        .find(|page| page.first <= expected.lineage_id().0 && expected.lineage_id().0 <= page.last)
        .unwrap();
    let receiver_medium = TestDurableMedium::new();
    let attachment = receiver_medium.attachment();
    for bytes in source.commits.values() {
        let artifact =
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
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
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    NodeFixture {
        runtime,
        source,
        expected,
        page,
        directory,
    }
}
