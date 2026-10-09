use crate::facade::identity::EntityId;
use crate::facade::transactions::{MutationIntent, TransactionCommitError};
use crate::tests::support::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};

#[test]
fn update_entity_fields_rejects_undeclared_aspect_targets() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "field-guard");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("update-fields-undeclared").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::transactions::data::AspectFieldPatch::from(
                    std::collections::BTreeMap::from([(
                        crate::transactions::data::planned_single_field_locator(
                            AspectKey::new("undeclared").expect("valid test aspect key"),
                            FieldKey::new("undeclared").unwrap(),
                        ),
                        AspectValue::String(InternedString::Raw("nope".to_string())),
                    )]),
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            match error.class {
                crate::transactions::data::ConflictClass::RecordAspectPatchDenied {
                    denial:
                        crate::transactions::data::RecordAspectPatchDenial::FieldAuthoringDenied {
                            ref target,
                            reason: crate::transactions::data::AspectFieldTargetRejectionReason::UndeclaredAspect,
                        },
                    ..
                } => assert_eq!(target.aspect().aspect_key().as_str(), "undeclared"),
                other => panic!("expected typed record aspect patch denial, got {other:?}"),
            }
            assert!(error.detail.contains("undeclared aspect"));
        }
        other => panic!("expected conflict error, got {other:?}"),
    }
}

#[test]
fn update_entity_fields_state_conflict_is_typed_not_json() {
    let entity_id = EntityId::new(PartitionId::main(), 7, 0);
    let conflict = crate::transactions::data::CommitConflict::new(
        crate::transactions::data::ConflictClass::EntityFieldUpdateStateInconsistency {
            entity_id,
            missing:
                crate::transactions::data::EntityFieldUpdateMissingState::AuthoritativeAspectState,
        },
    );

    match conflict.class {
        crate::transactions::data::ConflictClass::EntityFieldUpdateStateInconsistency {
            entity_id: actual_entity_id,
            missing,
        } => {
            assert_eq!(actual_entity_id, entity_id);
            assert_eq!(
                missing,
                crate::transactions::data::EntityFieldUpdateMissingState::AuthoritativeAspectState
            );
        }
        other => panic!("expected typed entity field update state conflict, got {other:?}"),
    }
    assert!(conflict
        .detail
        .contains("retained authoritative aspect state after stale-target validation"));
}

#[test]
fn update_entity_fields_rejects_explicit_aspect_field_path_mismatch() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("title.scalar"),
                crate::tests::support::field_key("title"),
            ),
            entity_summary_struct_aspect(
                crate::tests::support::aspect_key("summary"),
                crate::tests::support::field_key("summary"),
            ),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity = super::struct_field_patch_authority::create_entity_with_summary_fields(
        &runtime,
        "ambiguous-title",
        "before",
        "open",
        false,
        true,
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("mismatched-aspect-field").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        AspectKey::new("title.scalar").expect("valid test aspect key"),
                        FieldKey::new("status").expect("valid test field key"),
                    ),
                    AspectValue::String("after".into()),
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => match error.class {
            crate::transactions::data::ConflictClass::RecordAspectPatchDenied {
                denial:
                    crate::transactions::data::RecordAspectPatchDenial::FieldAuthoringDenied {
                        target: field_locator,
                        reason: crate::transactions::data::AspectFieldTargetRejectionReason::FieldPathNotAdmittedByAspectBinding,
                    },
                ..
            } => assert_eq!(field_locator.aspect().aspect_key().as_str(), "title.scalar"),
            other => panic!("expected entity aspect field path mismatch denial, got {other:?}"),
        },
        other => panic!("expected conflict error, got {other:?}"),
    }
}

#[test]
fn update_entity_fields_validation_denial_carries_aspect_field_path() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "type-denial");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("field-patch-type-denial").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        AspectKey::new("name").expect("valid test aspect key"),
                        FieldKey::new("name").expect("valid test field key"),
                    ),
                    AspectValue::UInt64(7),
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => match error.class {
            crate::transactions::data::ConflictClass::RecordAspectPatchDenied {
                denial:
                    crate::transactions::data::RecordAspectPatchDenial::ReadmissionDenied(
                        worth_foundational::facade::PortableAspectReadmissionDenial::ValueValidation {
                            key,
                            ..
                        },
                    ),
                ..
            } => {
                assert_eq!(key.as_str(), "name");
            }
            other => panic!("expected contract validation denial, got {other:?}"),
        },
        other => panic!("expected conflict error, got {other:?}"),
    }
}
