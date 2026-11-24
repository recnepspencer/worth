use super::*;
use crate::facade::history::BranchId;
use crate::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::facade::transactions::{
    EntityMutationIntent, MutationIntent, RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use crate::history::data::RelationalDescriptiveTouch as Touch;
use crate::storage::data::AuthoritativeFieldComparisonKey;

#[test]
fn native_merge_touches_only_changed_revision_and_both_index_membership_keys() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("status").unwrap(),
        field_key("status"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &mut runtime,
        "shared",
        crate::tests::support::single_string_aspect_field_patch(
            AspectKey::new("status").unwrap(),
            field_key("status"),
            "off",
        ),
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "status.merge.touch".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: crate::tests::support::aspect_field_locator(
                    AspectKey::new("status").unwrap(),
                    field_key("status"),
                ),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    // Branch-scoped indexes are built explicitly for each admitted basis: after
    // registration on main, and for the fork; ordinary maintenance refuses to
    // reconstruct them implicitly.
    rebuild_branch_index(execution, &runtime, "main", index.index_id);
    create_branch_from_main(&runtime, "feature");
    rebuild_branch_index(execution, &runtime, "feature", index.index_id);
    let mut source = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    source
        .push_batch(
            WorkerIntentBatch::new("source-status").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        AspectKey::new("status").unwrap(),
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
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
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
    let old = AuthoritativeFieldComparisonKey::from_aspect_value(
        &crate::tests::support::string_aspect_value("off"),
    );
    let new = AuthoritativeFieldComparisonKey::from_aspect_value(
        &crate::tests::support::string_aspect_value("active"),
    );
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
    let mut original_roots = Vec::new();
    for branch in [&main, &feature] {
        assert_eq!(
            read_entity_aspect_field_display(
                execution,
                &runtime,
                branch,
                entity,
                AspectKey::new("status").unwrap(),
                field_key("status"),
            ),
            "active",
            "the selected target must include the non-first-parent contribution"
        );
        let head = runtime.history().branch_head(branch).unwrap();
        let original = runtime.history.branch_cell(branch).unwrap().root().unwrap();
        assert_eq!(original.commit_id(), Some(head.commit_id));
        original_roots.push((head.commit_id, original.axes(), original));
        let built = runtime.index_authority().build_for_commit(
            execution,
            crate::indexes::data::DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: branch.clone(),
                index_ids: vec![index.index_id],
            },
        );
        assert!(built.failed_indexes.is_empty(), "{built:?}");
        assert_eq!(built.generations.len(), 1);
        let generation = built.generations.into_iter().next().unwrap();
        let crate::indexes::data::DerivedIndexEntries::EntityField(entries) = &generation.entries
        else {
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

    // This independent resident producer exports the actual original Root columns,
    // including history, revision stamps and peer metadata, before the bare fold.
    let original = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let original = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        original.bytes(),
    )
    .unwrap()
    .checkpoint;
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(
            execution,
            crate::durability::data::RelationalCheckpointArtifactPolicy::Embedded,
        )
        .expect("genuine complete multi-parent reconstruction");
    let complete = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    for (commit, axes, root) in original_roots {
        let expected = original
            .branch_roots
            .iter()
            .find(|image| image.commit_id == commit)
            .expect("original selected Root image");
        let actual = complete
            .branch_roots
            .iter()
            .find(|image| image.commit_id == commit)
            .expect("Complete must preserve both current and feature Roots");
        assert_eq!(actual, expected, "all original Root columns must agree");
        assert_eq!(root.commit_id(), Some(commit));
        assert_eq!(root.axes(), axes);
    }
    // Complete carries its generations as checkpoint deltas beside an empty
    // full-cache slot. Generations are immutable by id, and each one built above
    // was checked for its exact membership, so Complete preserves that membership
    // and its source axes when it binds that exact generation at each head.
    assert!(complete.derived_index_artifacts.generations().is_empty());
    let carried = complete
        .derived_index_checkpoint
        .as_ref()
        .expect("Complete carries its index checkpoint");
    for generation in &generations {
        assert!(
            carried.bases().iter().any(|basis| basis.branch == generation.source_branch_id
                && basis.commit == generation.source_commit_id
                && basis
                    .bindings
                    .contains(&(generation.index_id, generation.generation_id))),
            "Complete must preserve actual semantic membership and source axes:              {:?} at {:?} {:?} in {:?}",
            generation.generation_id,
            generation.source_branch_id,
            generation.source_commit_id,
            carried.bases()
        );
    }
}
