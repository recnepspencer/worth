use worth_foundational::facade::{ContractValidatedAspectValueView, FieldKey, StructAspectValue};
use worth_foundational::{
    AspectKey as FoundationalAspectKey, AspectValue as FoundationalAspectValue,
    InternedString as FoundationalInternedString,
};

use crate::authority::mutation::outcomes::RecordMutation;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::publication::patch::data::ordered_aspect_keys;
use crate::runtime::RuntimeSymbolTable;
use crate::schema::data::{AspectBinding, RelationalSchemaRegistry};
use crate::transactions::data::AspectTraceEvidence;

use super::super::canonical_delta_for_mutation;
use super::super::data::{AuthoritativePatchDeltaOperation, CanonicalAspectDeltaEvidence};
use super::support::{
    assert_authoritative_whole_aspect_locator, authoritative_string_patch,
    authoritative_summary_field_patch, authoritative_summary_patch, catalog_with_entity_binding,
    catalog_with_relation_binding, empty_working_state, empty_workspace, mutation_config,
    scalar_string_contract, summary_struct_contract,
};

#[test]
fn entity_field_delta_materializes_authoritative_aspect_patch() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let schema = RelationalSchemaRegistry::new();
    let contract = scalar_string_contract(FoundationalAspectKey::new("name").unwrap(), 1, 7);
    let authoritative_patch = authoritative_string_patch(&contract, "native-authority");
    let catalog = catalog_with_entity_binding(
        KindId(1),
        contract,
        AspectBinding::EntityField {
            field: worth_foundational::facade::FieldKey::new("name").expect("valid field"),
        },
    );
    let mutation = RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId(1), 0, 1),
        kind_id: KindId(1),
        authoritative_patch: Some(authoritative_patch),
    };

    let delta = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .expect("entity field evidence should come from authoritative patch");

    assert_eq!(
        delta.changed_aspects,
        ordered_aspect_keys([FoundationalAspectKey::new("name").unwrap()])
    );
    let CanonicalAspectDeltaEvidence::AuthoritativePatch {
        locator,
        operation: AuthoritativePatchDeltaOperation::WholeAspectSet { value },
    } = &delta.evaluated_bindings[0].evidence
    else {
        panic!("expected scalar authoritative patch evidence");
    };
    assert_authoritative_whole_aspect_locator(locator, "name");
    assert_eq!(value.key().as_str(), "name");
    let ContractValidatedAspectValueView::Scalar(FoundationalAspectValue::String(actual)) =
        value.view()
    else {
        panic!("expected validated scalar string authoritative patch value");
    };
    assert_eq!(
        actual,
        &FoundationalInternedString::Raw("native-authority".to_string())
    );
}

#[test]
fn entity_struct_delta_materializes_authoritative_patch_value() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let schema = RelationalSchemaRegistry::new();
    let contract = summary_struct_contract(FoundationalAspectKey::new("summary").unwrap());
    let authoritative_patch = authoritative_summary_patch(&contract, "native-summary");
    let catalog = catalog_with_entity_binding(
        KindId(1),
        contract,
        AspectBinding::EntityField {
            field: worth_foundational::facade::FieldKey::new("summary").expect("valid field"),
        },
    );
    let mutation = RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId(1), 0, 1),
        kind_id: KindId(1),
        authoritative_patch: Some(authoritative_patch),
    };

    let delta = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .expect("struct evidence should come from authoritative patch");

    let CanonicalAspectDeltaEvidence::AuthoritativePatch {
        operation: AuthoritativePatchDeltaOperation::WholeAspectSet { value },
        ..
    } = &delta.evaluated_bindings[0].evidence
    else {
        panic!("expected struct authoritative patch evidence");
    };
    assert_eq!(value.key().as_str(), "summary");
    let ContractValidatedAspectValueView::Struct(actual) = value.view() else {
        panic!("expected validated struct authoritative patch value");
    };
    assert_eq!(
        struct_field_value(actual, "title"),
        Some(&FoundationalAspectValue::String("native-summary".into()))
    );

    let trace = delta.evaluation_trace();
    let AspectTraceEvidence::AuthoritativePatch { locator, patch } =
        &trace.binding_rows[0].evidence
    else {
        panic!("expected struct authoritative patch trace evidence");
    };
    assert_authoritative_whole_aspect_locator(locator, "summary");
    let aspect_key = FoundationalAspectKey::new("summary").unwrap();
    let trace_value = patch
        .struct_set_for(&aspect_key)
        .expect("summary struct patch value");
    assert_eq!(
        struct_field_value(trace_value, "title"),
        Some(&FoundationalAspectValue::String("native-summary".into()))
    );
}

#[test]
fn entity_struct_delta_materializes_foundational_field_level_patch_evidence() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let schema = RelationalSchemaRegistry::new();
    let contract = summary_struct_contract(FoundationalAspectKey::new("summary").unwrap());
    let authoritative_patch = authoritative_summary_field_patch(&contract, "field-level-summary");
    let catalog = catalog_with_entity_binding(
        KindId(1),
        contract,
        AspectBinding::EntityField {
            field: worth_foundational::facade::FieldKey::new("summary").expect("valid field"),
        },
    );
    let mutation = RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId(1), 0, 1),
        kind_id: KindId(1),
        authoritative_patch: Some(authoritative_patch),
    };

    let delta = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .expect("field-level struct evidence should come from authoritative patch");

    let CanonicalAspectDeltaEvidence::AuthoritativePatch {
        operation: AuthoritativePatchDeltaOperation::FieldLevelPatch { patch },
        ..
    } = &delta.evaluated_bindings[0].evidence
    else {
        panic!("expected foundational field-level authoritative patch evidence");
    };
    assert_eq!(patch.key().as_str(), "summary");
    assert_eq!(
        patch
            .field_sets()
            .find(|(field, _)| field.as_str() == "title")
            .map(|(_, value)| value),
        Some(&FoundationalAspectValue::String(
            "field-level-summary".into()
        ))
    );

    let trace = delta.evaluation_trace();
    let AspectTraceEvidence::AuthoritativePatch { patch, .. } = &trace.binding_rows[0].evidence
    else {
        panic!("expected field-level authoritative patch trace evidence");
    };
    let aspect_key = FoundationalAspectKey::new("summary").unwrap();
    let title = FieldKey::new("title").expect("valid field");
    assert_eq!(
        patch
            .field_sets_for(&aspect_key)
            .map(|field_set| (&field_set.field, &field_set.value))
            .collect::<Vec<_>>(),
        vec![(
            &title,
            &FoundationalAspectValue::String("field-level-summary".into())
        )]
    );
}

#[test]
fn relation_field_delta_materializes_authoritative_aspect_patch() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let schema = RelationalSchemaRegistry::new();
    let contract =
        scalar_string_contract(FoundationalAspectKey::new("relation.label").unwrap(), 11, 3);
    let catalog = catalog_with_relation_binding(
        KindId(2),
        contract.clone(),
        AspectBinding::RelationField {
            field: worth_foundational::facade::FieldKey::new("label").expect("valid field"),
        },
    );
    let authoritative_patch = authoritative_string_patch(&contract, "native-authority");
    let source = EntityId::new(PartitionId(1), 0, 1);
    let target = EntityId::new(PartitionId(1), 1, 1);
    let mutation = RecordMutation::RelationCreated {
        relation_id: RelationId::new(PartitionId(2), 0, 1),
        kind_id: KindId(2),
        source,
        target,
        authoritative_patch: Some(authoritative_patch),
    };

    let delta = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .expect("relation field evidence should come from authoritative aspect patch");

    let CanonicalAspectDeltaEvidence::AuthoritativePatch {
        locator,
        operation: AuthoritativePatchDeltaOperation::WholeAspectSet { value },
    } = &delta.evaluated_bindings[0].evidence
    else {
        panic!("expected scalar authoritative relation patch evidence");
    };
    assert_authoritative_whole_aspect_locator(locator, "relation.label");
    assert_eq!(value.key().as_str(), "relation.label");
    let ContractValidatedAspectValueView::Scalar(FoundationalAspectValue::String(actual)) =
        value.view()
    else {
        panic!("expected validated scalar string authoritative patch value");
    };
    assert_eq!(
        actual,
        &FoundationalInternedString::Raw("native-authority".to_string())
    );
}

fn struct_field_value<'a>(
    struct_value: &'a StructAspectValue,
    field_name: &str,
) -> Option<&'a FoundationalAspectValue> {
    let field_key = FieldKey::new(field_name).expect("valid field");
    struct_value
        .fields()
        .find(|(field, _)| *field == &field_key)
        .map(|(_, value)| value)
}
