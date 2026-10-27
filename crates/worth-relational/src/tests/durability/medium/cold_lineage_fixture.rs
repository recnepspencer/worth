//! A producer-written cold directory with several actual lineage node pages.
use super::*;
use crate::durability::data::RelationalCheckpointPage;
use crate::lineage::data::LineageNode;

pub(super) const POPULATION: usize = 601;

pub(super) struct NodeFixture {
    pub(super) runtime: crate::runtime::RelationalRuntime,
    pub(super) source: Arc<MemberSource>,
    pub(super) expected: LineageNode,
    pub(super) page: RelationalCheckpointPage,
    pub(super) root_history: crate::durability::medium::RelationalDurableMemberLocator,
    pub(super) mirror_head: crate::durability::medium::RelationalDurableMemberLocator,
    pub(super) region_history: crate::durability::medium::RelationalDurableMemberLocator,
}

pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> NodeFixture {
    let producer = durable_runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&producer);
    transaction
        .push_batch(
            WorkerIntentBatch::new("paged-node-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(crate::transactions::data::BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..POPULATION)
                        .map(|id| crate::symbols::data::ClientKey::raw(format!("node-{id:04}")))
                        .collect(),
                    field_patches: (0..POPULATION)
                        .map(|_| name_field_patch("node-payload"))
                        .collect(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &producer).unwrap();
    let entity = *changed_entities(&committed).last().unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let expected = producer
        .lineage
        .nodes_snapshot()
        .into_iter()
        .find(|node| node.entity_id() == entity)
        .unwrap();
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
    source.damaged.store(false, Ordering::Relaxed);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Input>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    assert!(
        manifest.lineage.len() > 2,
        "the node population spans multiple real pages"
    );
    let page = *manifest
        .lineage
        .iter()
        .find(|page| page.first <= expected.lineage_id().0 && expected.lineage_id().0 <= page.last)
        .unwrap();
    let region_history = manifest
        .mirror
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .history;
    let attachment = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(attachment);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    assert_eq!(runtime.lineage.node_count(), 0);
    let selected = runtime
        .history()
        .branch_head(&crate::history::data::BranchId("main".into()))
        .unwrap();
    let root_history = manifest
        .roots
        .iter()
        .find(|root| root.commit_id == selected.commit_id)
        .unwrap()
        .regions
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .history;
    let mirror_head = manifest
        .mirror
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .head;
    NodeFixture {
        root_history,
        mirror_head,
        runtime,
        source,
        expected,
        page,
        region_history,
    }
}
