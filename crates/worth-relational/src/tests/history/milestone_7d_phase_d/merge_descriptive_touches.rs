use super::*;
use crate::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::history::data::RelationalDescriptiveTouch as Touch;
use crate::storage::data::AuthoritativeFieldComparisonKey;

#[test]
fn native_merge_touches_only_changed_revision_and_both_index_membership_keys() {
    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("status").unwrap(),
        field_key("status"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        &mut runtime,
        "shared",
        crate::tests::support::single_string_aspect_field_patch(
            AspectKey::new("status").unwrap(),
            field_key("status"),
            "off",
        ),
    );
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "status.merge.touch".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: crate::tests::support::aspect_field_locator(
                AspectKey::new("status").unwrap(),
                field_key("status"),
            ),
        },
        branch_scoped: true,
    });
    create_branch_from_main(&runtime, "feature");
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
        )
        .unwrap();
    source.commit(&runtime).expect("source status commit");

    let prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("native merge preparation");
    let merged = runtime
        .execute_prepared_merge(prepared)
        .expect("native merge commit");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(merged.commit.commit.commit_id)
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
}
