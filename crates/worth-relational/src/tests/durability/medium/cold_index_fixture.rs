//! Actual producer generation and external members, reopened without payload I/O.
use super::*;
use crate::durability::medium::RelationalDurableColdSource;
use crate::indexes::data::{
    DerivedIndexDefinition, DerivedIndexGeneration, DerivedIndexId, DerivedIndexKind,
};
pub(super) struct IndexFixture {
    pub(super) runtime: crate::runtime::RelationalRuntime,
    pub(super) source: Arc<MemberSource>,
    pub(super) entity: crate::identity::data::EntityId,
    pub(super) index: DerivedIndexId,
    pub(super) generation: Arc<DerivedIndexGeneration>,
    pub(super) member: RelationalDurableMemberLocator,
}
pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> IndexFixture {
    reopened_with_source(execution, |source| source)
}
pub(super) fn reopened_with_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    wrap: impl FnOnce(Arc<MemberSource>) -> Arc<dyn RelationalDurableColdSource>,
) -> IndexFixture {
    let producer =
        durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let index = producer
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "matrix-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    let entity = create_entity(execution, &producer, "matrix-index-value");
    let branch = crate::history::data::BranchId("main".into());
    let head = producer.history().branch_head(&branch).unwrap();
    let built = producer.index_authority().build_for_commit(
        execution,
        crate::indexes::data::DerivedIndexBuildRequest {
            source_commit_id: head.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index],
        },
    );
    assert!(
        built.failed_indexes.is_empty(),
        "the actual producer must build this index"
    );
    let generation = producer
        .index_access()
        .latest_generation(index, &branch)
        .unwrap();
    assert_eq!(generation.source_commit_id, head.commit_id);
    let (recovery, source, entity) = checkpoint_fixture(execution, producer, entity, false);
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
    assert!(manifest
        .index_definitions
        .iter()
        .any(|definition| definition.index_id == index));
    let attachment = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime =
        durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    bind_durable_medium(&mut runtime, attachment);
    let recovery = RelationalDurableRecoverySource::new(
        recovery.checkpoint().cloned(),
        recovery.tail().to_vec(),
        recovery.transitions().to_vec(),
        wrap(source.clone()),
    );
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
    IndexFixture {
        runtime,
        source,
        entity,
        index,
        generation,
        member: manifest.derived_indexes,
    }
}
