//! Genuine Native publication distinguishes a field clear from whole-aspect removal.
use super::*;
use crate::facade::{
    publication::PublishedAuthoritativeFieldEffect as Effect, runtime::RelationalFieldPresence,
};
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

#[test]
fn scalar_whole_writes_use_the_actual_binding_and_original_absence_revision() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let contract = optional_string_contract("optional_note");
    let binding = AspectBinding::EntityField {
        field: field_key("physical_note"),
    };
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: binding.clone(),
            contract: contract.clone(),
        }],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let created = commit_entity_create(execution, &runtime, whole_set(&contract, text("same")));
    let entity = changed_entities(&created)[0];
    let field = field_key("physical_note");
    let effect = |commit: &crate::facade::transactions::CommitResult| {
        commit
            .publication()
            .envelope
            .patch
            .authoritative_record_patches[0]
            .authoritative_patch
            .field_effect(&contract, &binding, &field, |_| Ok::<_, ()>(()))
            .unwrap()
    };
    let revision = |commit: &crate::facade::transactions::CommitResult| {
        runtime
            .read_truth()
            .project_snapshot(&commit.snapshot)
            .unwrap()
            .entity_field_revision(
                execution,
                entity,
                &AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    contract.key().clone(),
                    CanonicalFieldPath::single(field.clone()),
                ),
            )
            .unwrap()
            .unwrap()
    };
    let value = |commit: &crate::facade::transactions::CommitResult| {
        let patch = &commit
            .publication()
            .envelope
            .patch
            .authoritative_record_patches[0]
            .authoritative_patch;
        let (observed, location) = patch
            .field_effect_with_value_location(&contract, &binding, &field, |_| Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(observed, effect(commit));
        location
            .and_then(|location| patch.field_value_at(location))
            .cloned()
    };
    assert_eq!(effect(&created), Effect::Present);
    assert_eq!(value(&created), Some(text("same")));
    let cleared = commit_entity_patch(execution, &runtime, entity, whole_clear(&contract));
    assert_eq!(effect(&cleared), Effect::AspectCleared);
    assert_eq!(value(&cleared), None);
    assert_eq!(
        revision(&cleared).presence(),
        RelationalFieldPresence::Absent
    );
    assert_eq!(revision(&cleared).version(), cleared.commit.version_id);
    let present = commit_entity_patch(
        execution,
        &runtime,
        entity,
        whole_set(&contract, text("same")),
    );
    assert_eq!(effect(&present), Effect::Present);
    assert_eq!(value(&present), Some(text("same")));
    assert_eq!(
        value(&created),
        Some(text("same")),
        "later publications leave original scalar custody intact"
    );
    assert_eq!(
        revision(&present).presence(),
        RelationalFieldPresence::Present
    );
    assert_ne!(
        revision(&created),
        revision(&present),
        "same bytes are another occurrence"
    );
    let cleared_again = commit_entity_patch(execution, &runtime, entity, whole_clear(&contract));
    assert_eq!(effect(&cleared_again), Effect::AspectCleared);
    assert_ne!(
        revision(&cleared),
        revision(&cleared_again),
        "absence ABA is revised"
    );
    let guessed = AspectBinding::EntityField {
        field: field_key("optional_note"),
    };
    assert_eq!(
        runtime
            .read_truth()
            .project_snapshot(&cleared_again.snapshot)
            .unwrap()
            .entity_field_revision(
                execution,
                entity,
                &AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    contract.key().clone(),
                    CanonicalFieldPath::single(field_key("optional_note"))
                )
            )
            .unwrap(),
        None,
        "an undeclared scalar field is unavailable, never an absent revision"
    );
    assert_eq!(
        created
            .publication()
            .envelope
            .patch
            .authoritative_record_patches[0]
            .authoritative_patch
            .field_effect(&contract, &guessed, &field_key("optional_note"), |_| Ok::<
                _,
                (),
            >(
                ()
            ))
            .unwrap(),
        Effect::Unsupported
    );
    let patch = &created
        .publication()
        .envelope
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch;
    let (effect, location) = patch
        .field_effect_with_value_location(&contract, &guessed, &field_key("optional_note"), |_| {
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(effect, Effect::Unsupported);
    assert!(location.is_none());
    // Hostile decoded descriptive grammar is not a newly published commit.
    // Repeated writes must never issue a value location by guessing an order.
    let mut decoded = serde_json::to_value(patch).unwrap();
    let operations = decoded["operations"].as_array_mut().unwrap();
    operations.push(operations[0].clone());
    let repeated: crate::facade::publication::PublishedAuthoritativePatch =
        serde_json::from_value(decoded).unwrap();
    let (effect, location) = repeated
        .field_effect_with_value_location(&contract, &binding, &field, |_| Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(effect, Effect::Ambiguous);
    assert!(location.is_none());
    for commit in [&created, &cleared, &present, &cleared_again] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn struct_replacement_and_field_clear_preserve_distinct_canonical_effects() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let installed = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = installed.contract.clone();
    let binding = installed.binding.clone();
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![installed],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let field = field_key("status");
    let set = |status: bool| {
        let mut fields = vec![(field_key("title"), text("original"))];
        if status {
            fields.push((field.clone(), text("open")));
        }
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Struct(StructAspectValue::new(fields).unwrap()),
        }])
    };
    let created = commit_entity_create(execution, &runtime, set(true));
    let entity = changed_entities(&created)[0];
    let cleared = commit_entity_patch(
        execution,
        &runtime,
        entity,
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
            basis: PortableAspectContractBasis::from_contract(&contract),
            selected_fields: vec![field.clone()],
            field_sets: vec![],
            field_clears: vec![field.clone()],
        }]),
    );
    let restored = commit_entity_patch(execution, &runtime, entity, set(true));
    let replaced = commit_entity_patch(execution, &runtime, entity, set(false));
    for (commit, expected) in [
        (&created, Effect::Present),
        (&cleared, Effect::FieldCleared),
        (&restored, Effect::Present),
        (&replaced, Effect::FieldCleared),
    ] {
        assert_eq!(
            commit
                .publication()
                .envelope
                .patch
                .authoritative_record_patches[0]
                .authoritative_patch
                .field_effect(&contract, &binding, &field, |_| Ok::<_, ()>(()))
                .unwrap(),
            expected
        );
        let patch = &commit
            .publication()
            .envelope
            .patch
            .authoritative_record_patches[0]
            .authoritative_patch;
        let (effect, location) = patch
            .field_effect_with_value_location(&contract, &binding, &field, |_| Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(effect, expected);
        assert_eq!(
            location.and_then(|location| patch.field_value_at(location)),
            (expected == Effect::Present).then_some(&text("open"))
        );
    }
    // Cancellation belongs to the actual caller, before reading the canonical payload.
    assert_eq!(
        replaced
            .publication()
            .envelope
            .patch
            .authoritative_record_patches[0]
            .authoritative_patch
            .field_effect(&contract, &binding, &field, |_| Err("stopped")),
        Err("stopped")
    );
    for commit in [&created, &cleared, &restored, &replaced] {
        release_test_commit_snapshot(&runtime, commit);
    }
}
