//! Carried index selections survive cold reopen and checkpoint capture through public owners.
use super::{current_publications::create_entity, index_source::IndexRecipe};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::KindId,
    indexes::{
        BoundedIndexParityMode, DerivedIndexBuildRequest, DerivedIndexDefinition,
        DerivedIndexGeneration, DerivedIndexId, DerivedIndexKind,
    },
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_field_locator, aspect_key, field_key, publications::string_aspect_value,
    },
    cold_recovery::update_entity,
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};

/// Every expected generation is the one published for the main observation.
fn assert_selected(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    expected: &[&DerivedIndexGeneration],
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    for generation in expected {
        assert_eq!(
            runtime
                .index_access()
                .read_published_generation_for_observation(
                    request,
                    generation.index_id,
                    &basis.observation()
                )
                .unwrap()
                .as_deref(),
            Some(*generation)
        );
    }
}

/// The actual native capture restored into an independent receiver.
fn restored(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> RelationalRuntime {
    let image = runtime
        .durability_authority()
        .native_checkpoint(request, request.allocation_policy())
        .unwrap();
    let medium = TestDurableMedium::new();
    let mut receiver = durable_runtime(&medium);
    receiver
        .durability_recovery()
        .restore_native_checkpoint(request, &image)
        .unwrap();
    receiver
}

#[test]
fn cold_checkpoint_preserves_carried_index_selection_and_reads_postings_without_a_full_warmup() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = IndexRecipe::carried(execution).reopen(execution);
    let carried = fixture.carried.as_ref().unwrap();
    assert_selected(execution, &fixture.runtime, &[&*fixture.generation, &**carried]);
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let view = fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let access = fixture.runtime.index_access();
    let lookup = access
        .prepare_entity_field_lookup(
            &view,
            fixture.index,
            KindId(1),
            &aspect_field_locator(aspect_key("name"), field_key("name")),
            execution,
        )
        .unwrap();
    let outcome = lookup
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("matrix-index-updated"),
            2,
            BoundedIndexParityMode::Production,
            execution,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(outcome.candidate_entity_ids(), &[fixture.entity]);
}

#[test]
fn complete_checkpoint_saves_the_selection_carried_to_its_current_root() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
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
    let branch = BranchId("main".into());
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let built = producer.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: branch.clone(),
            index_ids: vec![changed, stable],
        },
    );
    drop(basis);
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
    update_entity(execution, &producer, entity, "complete-index-updated");
    let current = latest(changed);
    assert_eq!(latest(stable), carried);
    assert!(carried.applicability.version_id < current.applicability.version_id);
    let receiver = restored(execution, &producer);
    assert_selected(execution, &receiver, &[&*current, &*carried]);
}

#[test]
fn reclose_before_any_image_demand_keeps_the_selections_the_image_carried() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    // An actual reopened owner whose index image is installed and never demanded.
    let fixture = IndexRecipe::carried(execution).reopen(execution);
    let carried = fixture.carried.as_ref().unwrap();
    let receiver = restored(execution, &fixture.runtime);
    assert_selected(execution, &receiver, &[&*fixture.generation, &**carried]);
}
