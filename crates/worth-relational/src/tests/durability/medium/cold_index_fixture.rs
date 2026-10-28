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
    pub(super) manifest: crate::durability::data::RelationalCheckpointManifest,
    pub(super) history_symbol: Option<crate::symbols::data::Symbol>,
}
pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> IndexFixture {
    reopened_with_source(execution, |source| source)
}
pub(super) fn reopened_with_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    wrap: impl FnOnce(Arc<MemberSource>) -> Arc<dyn RelationalDurableColdSource>,
) -> IndexFixture {
    reopened_shape(execution, wrap, false)
}
pub(super) fn reopened_historical(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> IndexFixture {
    reopened_shape(execution, |source| source, true)
}
fn reopened_shape(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    wrap: impl FnOnce(Arc<MemberSource>) -> Arc<dyn RelationalDurableColdSource>,
    historical: bool,
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
        .expect("index fixture has identity capacity")
        .index_id;
    let (entity, history_symbol, _retained_history) = if historical {
        use super::cold_symbols::view_levels::{admit_population, publish_text, TextMutation};
        admit_population(execution, &producer);
        let (entity, before) = publish_text(
            execution,
            &producer,
            TextMutation::Create,
            "paged-view-value-0300",
        );
        let (_, source) = publish_text(
            execution,
            &producer,
            TextMutation::Update(entity),
            "paged-view-value-0600",
        );
        let symbol = producer
            .services
            .symbols
            .interner_snapshot()
            .symbol("paged-view-value-0300")
            .unwrap();
        (entity, Some(symbol), Some((before, source)))
    } else {
        (
            create_entity(execution, &producer, "matrix-index-value"),
            None,
            None,
        )
    };
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
        .read_latest_generation(execution, index, &branch)
        .unwrap()
        .unwrap();
    assert_eq!(generation.source_commit_id, head.commit_id);
    if historical {
        update_entity_and_release_snapshot(execution, &producer, entity, "matrix-index-after");
    }
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
        manifest,
        history_symbol,
    }
}
