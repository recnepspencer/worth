//! A reconciling merge's descriptive touches, index membership keys and built generations.
use super::branch_history_publications::{fork, transaction};
use super::branch_history_recovery::current_head;
use super::merge_execution_authoring::{
    create_entity_with_aspect_fields, single_string_aspect_field_patch,
};
use super::merge_execution_observation::read_entity_aspect_field_display;
use super::merge_execution_schema::runtime_with_aspect_field_merge_policy;
use super::merge_proof_journey::merge_request;
use worth_relational::facade::durability::RelationalCheckpointArtifactPolicy;
use worth_relational::facade::history::{BranchId, RelationalDescriptiveTouch as Touch};
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId,
    DerivedIndexKind,
};
use worth_relational::facade::merge::AspectMergePolicyKind;
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::definition::{
    aspect_field_locator, aspect_key, field_key,
};
use worth_testkit_relational::aspect_storage::publications::string_aspect_value;

#[test]
fn native_merge_touches_only_changed_revision_and_both_index_membership_keys() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_aspect_field_merge_policy(
        aspect_key("status"),
        field_key("status"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &runtime,
        "shared",
        single_string_aspect_field_patch(aspect_key("status"), field_key("status"), "off"),
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "status.merge.touch".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("status"), field_key("status")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    fork(execution, &runtime, "feature");
    let mut source = transaction(&runtime, &BranchId("feature".to_string()));
    source
        .push_batch(
            WorkerIntentBatch::new("source-status").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("status"),
                        field_key("status"),
                        "active",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    source
        .commit(execution, &runtime)
        .expect("source status commit");

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("native merge preparation");
    let merged = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("native merge commit");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merged.commit.commit.commit_id)
        .unwrap()
        .expect("sealed canonical merge envelope");
    let touches = envelope
        .descriptive_touches()
        .exact_touches()
        .expect("native materialized merge has exact revisions");
    assert!(touches
        .iter()
        .any(|touch| matches!(touch, Touch::AspectRevision {
        record: RecordRef::Entity(id), aspect,
    } if *id == entity && aspect.as_str() == "status")));
    assert!(touches
        .iter()
        .any(|touch| matches!(touch, Touch::FieldRevision {
        record: RecordRef::Entity(id), aspect, path, ..
    } if *id == entity && aspect.as_str() == "status"
        && path.fields() == [field_key("status")])));
    assert!(
        !touches.iter().any(|touch| matches!(touch,
        Touch::AspectRevision { aspect, .. } | Touch::FieldRevision { aspect, .. }
            if aspect.as_str() == "name")),
        "retained A has no new revision: {touches:?}"
    );
    let old = AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value("off"));
    let new = AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value("active"));
    assert!(
        touches
            .iter()
            .any(|touch| matches!(touch, Touch::IndexMembership {
        index: id, old_key: Some(before), new_key: Some(after), ..
    } if *id == index.index_id && *before == old && *after == new)),
        "merge carries the actual old and new index membership: {touches:?}"
    );

    let main = BranchId("main".into());
    let feature = BranchId("feature".into());
    let mut generations = Vec::new();
    for branch in [&main, &feature] {
        assert_eq!(
            read_entity_aspect_field_display(
                execution,
                &runtime,
                branch,
                entity,
                aspect_key("status"),
                field_key("status"),
            ),
            "active",
            "the selected target must include the non-first-parent contribution"
        );
        let head = current_head(&runtime, branch).unwrap();
        let built = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: branch.clone(),
                index_ids: vec![index.index_id],
            },
        );
        assert!(built.failed_indexes.is_empty(), "{built:?}");
        assert_eq!(built.generations.len(), 1);
        let generation = built.generations.into_iter().next().unwrap();
        let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
            panic!("actual status field generation required")
        };
        assert_eq!(
            entries
                .get(&new)
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![entity]
        );
        assert!(entries.get(&old).is_none());
        generations.push(generation);
    }

    // The Complete all-root checkpoint is read back through the public restore
    // door: a separate Memory owner of the same declared schema must serve
    // exactly the generations built above, source axes included.
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(execution, RelationalCheckpointArtifactPolicy::Embedded)
        .expect("genuine complete multi-parent reconstruction");
    let mut restored = runtime_with_aspect_field_merge_policy(
        aspect_key("status"),
        field_key("status"),
        AspectMergePolicyKind::PreferRicher,
    );
    restored
        .durability_recovery()
        .restore_native_checkpoint(execution, complete.checkpoint())
        .expect("the Complete checkpoint restores a separate owner");
    let preserved = restored
        .index_access()
        .read_generations_for_version(execution, merged.commit.commit.version_id)
        .unwrap();
    for generation in generations {
        assert_eq!(
            preserved
                .iter()
                .find(|actual| actual.generation_id == generation.generation_id),
            Some(&generation),
            "the complete cache must preserve actual semantic membership and source axes"
        );
    }
}
