use super::*;

#[test]
fn scalar_set_query_ignores_field_level_patch_values_for_same_aspect() {
    let aspect_key = AspectKey::new("counter").unwrap();
    let field = FieldKey::new("value").unwrap();
    let patch = PublishedAuthoritativePatch::new(vec![
        PublishedAuthoritativePatchOperation::FieldLevelPatch {
            aspect_key: aspect_key.clone(),
            aspect_identity: AspectIdentity(1),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("counter").unwrap(),
            },
            field_sets: vec![PublishedAuthoritativeFieldSet {
                field,
                value: AspectValue::Int64(99),
            }],
            field_clears: Vec::new(),
        },
    ]);

    assert_eq!(patch.scalar_set_for(&aspect_key), None);
    let field_sets = patch.field_sets_for(&aspect_key).collect::<Vec<_>>();
    assert_eq!(field_sets[0].value, AspectValue::Int64(99));
}

#[test]
fn shape_specific_patch_queries_keep_operation_lanes_distinct() {
    let scalar_key = AspectKey::new("counter").unwrap();
    let struct_key = AspectKey::new("profile").unwrap();
    let clear_key = AspectKey::new("retired").unwrap();
    let patch = PublishedAuthoritativePatch::new(vec![
        PublishedAuthoritativePatchOperation::WholeAspectSet {
            aspect_key: scalar_key.clone(),
            aspect_identity: AspectIdentity(1),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("counter").unwrap(),
            },
            value: PublishedAuthoritativePatchValue::Scalar(AspectValue::Int64(7)),
        },
        PublishedAuthoritativePatchOperation::WholeAspectSet {
            aspect_key: struct_key.clone(),
            aspect_identity: AspectIdentity(2),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("profile").unwrap(),
            },
            value: PublishedAuthoritativePatchValue::Struct(
                StructAspectValue::new(Vec::<(FieldKey, AspectValue)>::new()).unwrap(),
            ),
        },
        PublishedAuthoritativePatchOperation::WholeAspectClear {
            aspect_key: clear_key.clone(),
            aspect_identity: AspectIdentity(3),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("retired").unwrap(),
            },
        },
    ]);

    assert_eq!(
        patch.scalar_set_for(&scalar_key),
        Some(&AspectValue::Int64(7))
    );
    assert!(patch.struct_set_for(&struct_key).is_some());
    assert!(patch.whole_clear_for(&clear_key));
    assert_eq!(patch.field_sets_for(&scalar_key).count(), 0);
}

#[test]
fn field_patch_queries_cover_every_operation_for_requested_aspect() {
    let aspect_key = AspectKey::new("summary").unwrap();
    let sibling_key = AspectKey::new("other").unwrap();
    let title = FieldKey::new("title").unwrap();
    let status = FieldKey::new("status").unwrap();
    let stale = FieldKey::new("stale").unwrap();
    let patch = PublishedAuthoritativePatch::new(vec![
        PublishedAuthoritativePatchOperation::FieldLevelPatch {
            aspect_key: aspect_key.clone(),
            aspect_identity: AspectIdentity(1),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("summary").unwrap(),
            },
            field_sets: vec![PublishedAuthoritativeFieldSet {
                field: title.clone(),
                value: AspectValue::String("draft".into()),
            }],
            field_clears: vec![],
        },
        PublishedAuthoritativePatchOperation::FieldLevelPatch {
            aspect_key: sibling_key,
            aspect_identity: AspectIdentity(2),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("other").unwrap(),
            },
            field_sets: vec![PublishedAuthoritativeFieldSet {
                field: FieldKey::new("ignored").unwrap(),
                value: AspectValue::String("ignored".into()),
            }],
            field_clears: vec![FieldKey::new("ignored_clear").unwrap()],
        },
        PublishedAuthoritativePatchOperation::FieldLevelPatch {
            aspect_key: aspect_key.clone(),
            aspect_identity: AspectIdentity(1),
            contract_revision: AspectContractRevision(1),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("summary").unwrap(),
            },
            field_sets: vec![PublishedAuthoritativeFieldSet {
                field: status.clone(),
                value: AspectValue::String("ready".into()),
            }],
            field_clears: vec![stale.clone()],
        },
    ]);

    let field_sets = patch
        .field_sets_for(&aspect_key)
        .map(|field_set| (&field_set.field, &field_set.value))
        .collect::<Vec<_>>();
    let field_clears = patch.field_clears_for(&aspect_key).collect::<Vec<_>>();

    assert_eq!(
        field_sets,
        vec![
            (&title, &AspectValue::String("draft".into())),
            (&status, &AspectValue::String("ready".into()))
        ]
    );
    assert_eq!(field_clears, vec![&stale]);
}
