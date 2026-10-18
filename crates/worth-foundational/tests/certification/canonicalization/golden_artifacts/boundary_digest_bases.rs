use serde_json::json;
use worth_foundational::{
    aspect_contract_digest_preparation_basis, aspect_mask_digest_preparation_basis,
    aspect_patch_digest_preparation_basis, aspect_state_digest_preparation_basis,
    lower_json_record_aspect_state, validate_aspect_value, AspectLocator, AspectMask, AspectValue,
    CanonicalDigestPreparationEntry, CanonicalFieldPath, JsonCompatibilityAspectInput,
    LocatorAuthority, MutationMask, StructAspectValue,
};
use worth_proof::TransitionOutcome;

use super::golden_fixtures::{
    admitted_state, golden_contract, golden_contract_basis, golden_mutation_mask_basis,
    golden_patch_basis, ready_contract, ready_mask, ready_patch, ready_state,
};
use crate::certification::foundational_vocabulary::{field, key};

#[test]
fn contract_mask_patch_state_and_compatibility_digest_basis_golden_is_semantic() {
    let contract = golden_contract();
    let mask = AspectMask::<MutationMask>::new([
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("title")),
    ]);
    let value = StructAspectValue::new([
        (field("title"), AspectValue::String("Ship it".into())),
        (field("done"), AspectValue::Bool(true)),
    ])
    .expect("unique fields");
    let TransitionOutcome::Success(entry) = validate_aspect_value(&contract, value.into()) else {
        panic!("expected native validation");
    };
    let TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([entry.clone()], [])
    else {
        panic!("expected whole aspect patch");
    };
    let json_state = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        contract.clone(),
        worth_foundational::BoundarySourceLocator::aspect(AspectLocator::new(
            LocatorAuthority::SupportOnly,
            key("task.summary"),
        )),
        json!({ "title": "Ship it", "done": true }),
    )]);
    let TransitionOutcome::Success(json_state) = json_state else {
        panic!("expected compatibility lowering");
    };

    assert_eq!(
        aspect_contract_digest_preparation_basis(&ready_contract(contract)),
        golden_contract_basis()
    );
    assert_eq!(
        aspect_mask_digest_preparation_basis(&ready_mask(key("task.summary"), mask)),
        golden_mutation_mask_basis()
    );
    assert_eq!(
        aspect_state_digest_preparation_basis(&ready_state(admitted_state([entry]))),
        aspect_state_digest_preparation_basis(&ready_state(json_state))
    );
    assert_eq!(
        aspect_patch_digest_preparation_basis(&ready_patch(&patch)),
        golden_patch_basis()
    );
}

#[allow(dead_code)]
fn assert_digest_basis(_basis: &[CanonicalDigestPreparationEntry]) {}
