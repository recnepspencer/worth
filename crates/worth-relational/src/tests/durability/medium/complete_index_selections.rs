//! The Complete producer saves the index selections carried to each root it emits.
use super::*;
use crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};

#[test]
fn complete_checkpoint_saves_the_selection_carried_to_its_current_root() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let producer =
        durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let register = |name: &str, aspect: &str| {
        producer
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: name.into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key(aspect), field_key(aspect)),
                },
            })
            .expect("index fixture has identity capacity")
            .index_id
    };
    let changed = register("complete-name", "name");
    let stable = register("complete-lifecycle", "lifecycle");
    let entity = create_entity(execution, &producer, "complete-index-value");
    let branch = crate::history::data::BranchId("main".into());
    let built = producer.index_authority().build_for_commit(
        execution,
        crate::indexes::data::DerivedIndexBuildRequest {
            source_commit_id: producer.history().branch_head(&branch).unwrap().commit_id,
            branch_id: branch.clone(),
            index_ids: [changed, stable].into_iter().collect(),
        },
    );
    assert!(
        built.failed_indexes.is_empty(),
        "the actual producer must build both indexes"
    );
    let latest = |index| {
        producer
            .index_access()
            .read_latest_generation(execution, index, &branch)
            .unwrap()
            .unwrap()
    };
    let carried = latest(stable);
    // Only the name index changes: the lifecycle selection is carried forward.
    update_entity_and_release_snapshot(execution, &producer, entity, "complete-index-updated");
    let current = latest(changed);
    assert_eq!(latest(stable), carried);
    assert!(carried.applicability.version_id < current.applicability.version_id);
    let head = producer.history().branch_head(&branch).unwrap();
    let complete = producer
        .complete_checkpoint_service()
        .prepare(
            execution,
            crate::durability::data::RelationalCheckpointArtifactPolicy::Embedded,
        )
        .unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    assert_eq!(
        decoded.derived_index_checkpoint_format,
        DerivedIndexCheckpointArtifacts::FORMAT_VERSION
    );
    assert!(decoded.derived_index_artifacts.is_empty());
    let saved = decoded
        .derived_index_checkpoint
        .as_ref()
        .expect("the Complete producer emits the selection-carrying payload");
    for basis in saved.bases() {
        assert!(
            decoded
                .branch_roots
                .iter()
                .any(|root| root.commit_id == basis.commit),
            "a saved selection always sits beside its emitted root image"
        );
    }
    let basis = saved
        .bases()
        .iter()
        .find(|basis| basis.branch == branch && basis.commit == head.commit_id)
        .expect("the current root keeps its selection");
    assert_eq!(basis.version, current.applicability.version_id);
    assert_eq!(
        basis.bindings,
        vec![
            (changed, current.generation_id),
            (stable, carried.generation_id)
        ]
    );
    let restored = saved
        .readmit(&crate::storage::substrate::maintenance::ReconstructionLane::oracle())
        .unwrap();
    for expected in [&carried, &current] {
        assert!(restored
            .iter()
            .any(|generation| generation == expected.as_ref()));
    }
}

#[test]
fn reclose_before_any_image_demand_keeps_the_selections_the_image_carried() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    // An actual reopened owner whose index image is installed and never demanded.
    let fixture = super::cold_index_fixture::reopened_carried(execution);
    let pending = fixture.runtime.indexes.carried_checkpoint_image();
    assert_eq!(pending, Some(fixture.member));
    let original = fixture
        .runtime
        .indexes
        .checkpoint_inputs_for_request(execution)
        .unwrap()
        .resolve_for_request(execution)
        .unwrap();
    assert_eq!(
        fixture.runtime.indexes.carried_checkpoint_image(),
        pending,
        "capture reads the original image without admitting it"
    );
    let carried = fixture.carried.as_ref().unwrap();
    let head = original
        .bases()
        .iter()
        .find(|basis| basis.version == fixture.generation.applicability.version_id)
        .expect("the image's current-root selection is carried into the next capture");
    assert_eq!(head.commit, fixture.generation.source_commit_id);
    let mut expected = vec![
        (fixture.index, fixture.generation.generation_id),
        (carried.index_id, carried.generation_id),
    ];
    expected.sort_unstable();
    assert_eq!(head.bindings, expected);
    for generation in [&fixture.generation, carried] {
        assert!(original
            .generations()
            .iter()
            .any(|emitted| emitted == generation));
    }
}
