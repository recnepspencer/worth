//! Exact index generations and query results survive actual external source recovery.
use super::checkpoint_recovery_observation::snapshot_for_branch;
use super::*;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{history::BranchId, indexes::*};
use worth_testkit_relational::aspect_storage::{
    definition::{aspect_field_locator, aspect_key, field_key},
    publications::changed_entities,
};
fn string_aspect_value(value: &str) -> AspectValue {
    AspectValue::String(value.into())
}
#[test]
fn restored_catalog_allocates_fresh_identities_and_preserves_old_observations() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &runtime, "before-recovery");
    let entity = changed_entities(&created)[0];
    let field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let mut definition = DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.before-recovery".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field.clone(),
        },
        branch_scoped: true,
    };
    // More than one definition/generation exposes reset-to-zero and reset-to-one.
    let first = runtime
        .index_authority()
        .register(definition.clone())
        .expect("index fixture has identity capacity");
    definition.name = "entity.name.second".into();
    let second = runtime
        .index_authority()
        .register(definition.clone())
        .expect("index fixture has identity capacity");
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: created.commit.branch_id.clone(),
            index_ids: vec![first.index_id, second.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    medium.file_checkpoint(capture(execution, &runtime));
    let (recovered, _) = recover(execution, &medium);
    let old_snapshot = snapshot_for_branch(&recovered, &BranchId("main".into()));
    definition.name = "entity.name.after-recovery".into();
    let third = recovered
        .index_authority()
        .register(definition.clone())
        .expect("index fixture has identity capacity");
    let fourth = recovered
        .index_authority()
        .register(definition)
        .expect("index fixture has identity capacity");
    assert!(third.index_id > second.index_id);
    assert!(fourth.index_id > third.index_id);
    let committed = super::checkpoint_index_generations::update_entity_on_branch(
        execution,
        &recovered,
        entity,
        "after-recovery",
        BranchId("main".into()),
    );
    let after = recovered.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: committed.commit.branch_id.clone(),
            index_ids: vec![
                first.index_id,
                second.index_id,
                third.index_id,
                fourth.index_id,
            ],
        },
    );
    assert!(after.failed_indexes.is_empty());
    let previous_max = build
        .generations
        .iter()
        .map(|generation| generation.generation_id)
        .max()
        .unwrap();
    assert!(after
        .generations
        .iter()
        .all(|generation| generation.generation_id > previous_max));
    for (snapshot, value, expected) in [
        (
            old_snapshot,
            "before-recovery",
            build.generations[0].generation_id,
        ),
        (
            committed.snapshot.clone(),
            "after-recovery",
            after.generations[0].generation_id,
        ),
    ] {
        let observed = recovered
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    snapshot,
                    first.index_id,
                    KindId(1),
                    field.clone(),
                    string_aspect_value(value),
                    1,
                )
                .unwrap(),
                BoundedIndexParityMode::Certification,
                execution,
            )
            .unwrap();
        assert_eq!(observed.generation_id(), expected);
        assert_eq!(observed.candidate_entity_ids(), &[entity]);
    }
}
