//! A writer reopened on the exact checkpoint and canonical journal prefix.
use super::*;
use std::collections::BTreeSet;

pub(super) struct CurrentFixture {
    pub(super) runtime: crate::runtime::RelationalRuntime,
    pub(super) source: Arc<MemberSource>,
    pub(super) entity: EntityId,
    pub(super) relation: RelationId,
    pub(super) lineage: crate::identity::data::LineageId,
    pub(super) heads: BTreeSet<RelationalDurableMemberLocator>,
    pub(super) symbols: BTreeSet<RelationalDurableMemberLocator>,
    pub(super) history: BTreeSet<RelationalDurableMemberLocator>,
}
impl CurrentFixture {
    pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let producer = durable_runtime_with_test_schema();
        let entity = create_entity(execution, &producer, "head-only-before");
        let target = create_entity(execution, &producer, "head-only-target");
        let linked =
            create_relation_outcome(execution, &producer, entity, target, "head-only-edge");
        let relation = changed_relations(&linked)[0];
        release_test_commit_snapshot(&producer, &linked);
        update_entity_and_release_snapshot(execution, &producer, entity, "head-only-current");
        let lineage = producer
            .lineage
            .nodes_snapshot()
            .into_iter()
            .find(|node| node.entity_id() == entity)
            .unwrap()
            .lineage_id();
        let (recovery, source, _) = checkpoint_fixture(execution, producer, entity, false);
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
        let history = manifest
            .mirror
            .iter()
            .chain(manifest.roots.iter().flat_map(|root| root.regions.iter()))
            .filter(|region| region.partition_id == entity.partition_id)
            .map(|region| region.history)
            .collect();
        let heads = manifest
            .mirror
            .iter()
            .chain(manifest.roots.iter().flat_map(|root| root.regions.iter()))
            .filter(|region| region.partition_id == entity.partition_id)
            .map(|region| region.head)
            .collect();
        let symbols = manifest.symbols.iter().map(|page| page.member).collect();
        let attachment = TestDurableMedium::new().attachment();
        for bytes in source.commits.values() {
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                attachment.publish(&artifact),
                crate::durability::medium::RelationalDurablePublication::Committed
            ));
        }
        let mut runtime = durable_runtime_with_test_schema();
        bind_durable_medium(&mut runtime, attachment);
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
        Self {
            runtime,
            source,
            entity,
            relation,
            lineage,
            history,
            heads,
            symbols,
        }
    }
}
