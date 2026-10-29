use worth_foundational::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy, AspectLocator, AspectValue,
    LocatorAuthority, OpaqueAspectType, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

#[test]
fn aspect_front_doors_cover_scalar_struct_mask_and_patch_progression() {
    let vocabulary = aspects().vocabulary();
    let title_key = vocabulary.field_key("title").expect("valid field");
    let label_key = vocabulary.field_key("label").expect("valid field");
    let task_key = vocabulary.key("task.summary").expect("valid aspect key");
    let field_path = vocabulary.field_path(["label"]).expect("single field path");

    assert_eq!(field_path.fields(), std::slice::from_ref(&label_key));

    let struct_shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .optional("label", ScalarAspectType::String)
        .optional("note", ScalarAspectType::String)
        .finish()
        .expect("struct shape");
    let struct_masks = aspects().mask_contract().struct_fields();
    let contract = aspects()
        .contract()
        .for_key(task_key.clone())
        .identified_by(vocabulary.identity(41))
        .at_revision(vocabulary.revision(1))
        .struct_with(
            struct_shape,
            struct_masks.clone(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        );

    assert_eq!(contract.masks(), &struct_masks);

    let projection = aspects()
        .projection_mask()
        .fields(["title", "label", "note"])
        .expect("projection mask");
    let mutation = aspects()
        .mutation_mask()
        .fields(["label", "note"])
        .expect("mutation mask");
    let diagnostic = aspects()
        .diagnostic_mask()
        .fields(["title", "label", "note"])
        .expect("diagnostic mask");

    assert!(contract.admits_projection_mask(&projection).is_ok());
    assert!(contract.admits_mutation_mask(&mutation).is_ok());
    assert!(contract.admits_diagnostic_mask(&diagnostic).is_ok());

    let struct_value = vocabulary
        .struct_value()
        .with_field("title", AspectValue::String("Ship it".into()))
        .with_field("label", AspectValue::String("origin".into()))
        .with_field("note", AspectValue::String("keep me".into()))
        .finish()
        .expect("struct value");
    let TransitionOutcome::Success(validated) =
        aspects().validate().against(&contract).value(struct_value)
    else {
        panic!("expected validated struct value");
    };
    let TransitionOutcome::Success(state) = aspects().authoritative_state().admit([validated])
    else {
        panic!("expected admitted state");
    };

    assert!(state.payload().get(&task_key).is_some());

    let TransitionOutcome::Success(field_patch) = aspects()
        .patch()
        .field_level(&contract, &mutation)
        .set_field(label_key.clone(), AspectValue::String("moved".into()))
        .clear_field(vocabulary.field_key("note").expect("valid field"))
        .finish()
    else {
        panic!("expected field-level patch");
    };

    let mut field_patches = field_patch.field_patches();
    let (patch_key, patch_body) = field_patches.next().expect("field patch entry");
    assert_eq!(patch_key, &task_key);
    assert_eq!(patch_body.key(), &task_key);
    assert!(patch_body.field_sets().any(|(key, _)| key == &label_key));
    assert!(patch_body.field_clears().any(|key| key.as_str() == "note"));
    assert!(field_patches.next().is_none());

    let TransitionOutcome::Success(applied_state) = aspects()
        .authoritative_state()
        .apply_patch(state.payload(), &field_patch)
    else {
        panic!("expected applied patch");
    };
    let patched_entry = applied_state
        .payload()
        .get(&task_key)
        .expect("patched aspect entry");
    let worth_foundational::ContractValidatedAspectValueView::Struct(patched_struct) =
        patched_entry.view()
    else {
        panic!("expected patched struct value");
    };
    assert_eq!(
        patched_struct.get(&label_key),
        Some(&AspectValue::String("moved".into()))
    );
    assert!(patched_struct
        .get(&vocabulary.field_key("note").expect("valid field"))
        .is_none());

    let aspect_locator = AspectLocator::new(LocatorAuthority::SupportOnly, task_key.clone());
    assert_eq!(aspect_locator.aspect_key(), &task_key);
    assert_eq!(title_key.as_str(), "title");
}

#[test]
fn vocabulary_field_paths_fail_closed_on_multi_segment_common_path_authoring() {
    let denial = aspects()
        .vocabulary()
        .field_path(["parent", "child"])
        .expect_err("front-door field paths are single-field Milestone 1 targeting");

    assert_eq!(
        denial,
        worth_foundational::AspectFrontDoorConstructionDenial::FieldPathMustTargetSingleField
    );
}

#[test]
fn aspect_front_doors_keep_scalar_validation_and_whole_patch_first_class() {
    let vocabulary = aspects().vocabulary();
    let retries_key = vocabulary.key("retry.count").expect("valid aspect key");
    let contract = aspects()
        .contract()
        .for_key(retries_key.clone())
        .identified_by(vocabulary.identity(7))
        .at_revision(vocabulary.revision(1))
        .scalar_with(
            ScalarAspectType::Int64,
            aspects().mask_contract().scalar(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        );

    let TransitionOutcome::Success(validated) = aspects()
        .validate()
        .against(&contract)
        .value(AspectValue::Int64(3))
    else {
        panic!("expected validated scalar value");
    };

    let TransitionOutcome::Success(patch) =
        aspects().patch().whole_aspect().set(validated).finish()
    else {
        panic!("expected whole-aspect patch");
    };

    assert!(patch
        .whole_aspect_sets()
        .any(|(key, _)| key == &retries_key));
    assert!(patch.whole_aspect_clears().next().is_none());

    let opaque_contract = aspects()
        .contract()
        .for_key(vocabulary.key("opaque.trace").expect("valid opaque key"))
        .identified_by(vocabulary.identity(8))
        .at_revision(vocabulary.revision(1))
        .opaque_with(
            OpaqueAspectType::Token,
            aspects().mask_contract().opaque_diagnostic_only(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::OpaqueIdentity,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("opaque contract");

    assert_eq!(
        opaque_contract.masks(),
        &aspects().mask_contract().opaque_diagnostic_only()
    );
}

#[test]
fn authoritative_state_front_door_rejects_empty_admission_requests() {
    let outcome = aspects().authoritative_state().admit([]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            worth_foundational::AuthoritativeStateAdmissionDenial::EmptyAdmission
        )
    );
}

#[test]
fn opaque_front_door_rejects_non_diagnostic_mask_contracts() {
    let vocabulary = aspects().vocabulary();

    let denial = aspects()
        .contract()
        .for_key(vocabulary.key("opaque.trace").expect("valid opaque key"))
        .identified_by(vocabulary.identity(8))
        .at_revision(vocabulary.revision(1))
        .opaque_with(
            OpaqueAspectType::Token,
            aspects().mask_contract().scalar(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::OpaqueIdentity,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect_err("opaque contracts must preserve diagnostic-only mask law");

    assert_eq!(
        denial,
        worth_foundational::AspectFrontDoorConstructionDenial::OpaqueMaskContractMustBeDiagnosticOnly
    );
}

#[test]
fn patch_front_door_rejects_empty_field_level_patch_requests() {
    let vocabulary = aspects().vocabulary();
    let contract = aspects()
        .contract()
        .for_key(vocabulary.key("task.summary").expect("valid aspect key"))
        .identified_by(vocabulary.identity(81))
        .at_revision(vocabulary.revision(1))
        .struct_with(
            aspects()
                .struct_fields()
                .required("title", ScalarAspectType::String)
                .optional("note", ScalarAspectType::String)
                .finish()
                .expect("struct shape"),
            aspects().mask_contract().struct_fields(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        );
    let mutation_mask = aspects()
        .mutation_mask()
        .fields(["note"])
        .expect("mutation mask");

    let outcome = aspects()
        .patch()
        .field_level(&contract, &mutation_mask)
        .finish();

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::EmptyFieldPatch
        )
    );
}
