#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::golden_artifacts::boundary_digest_bases::*;
use {serde_json::json};
use {worth_foundational::aspect_contract_digest_preparation_basis, worth_foundational::aspect_mask_digest_preparation_basis, worth_foundational::aspect_patch_digest_preparation_basis, worth_foundational::aspect_state_digest_preparation_basis, worth_foundational::lower_json_record_aspect_state, worth_foundational::validate_aspect_value, worth_foundational::AspectLocator, worth_foundational::AspectMask, worth_foundational::AspectValue, worth_foundational::CanonicalDigestPreparationEntry, worth_foundational::CanonicalFieldPath, worth_foundational::JsonCompatibilityAspectInput, worth_foundational::LocatorAuthority, worth_foundational::MutationMask, worth_foundational::StructAspectValue};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::admitted_state, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_contract, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_contract_basis, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_mutation_mask_basis, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_patch_basis, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_contract, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_mask, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_patch, crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_state};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key};

#[test]
fn contract_mask_patch_state_and_compatibility_digest_basis_golden_is_semantic() {
    let contract = golden_contract();
    let mask = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("title")),
    ]);
    let value = worth_foundational::StructAspectValue::new([
        (field("title"), worth_foundational::AspectValue::String("Ship it".into())),
        (field("done"), worth_foundational::AspectValue::Bool(true)),
    ])
    .expect("unique fields");
    let worth_proof::TransitionOutcome::Success(entry) = validate_aspect_value(&contract, value.into()) else {
        panic!("expected native validation");
    };
    let worth_proof::TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([entry.clone()], [])
    else {
        panic!("expected whole aspect patch");
    };
    let json_state = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        contract.clone(),
        worth_foundational::BoundarySourceLocator::aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::SupportOnly,
            key("task.summary"),
        )),
        json!({ "title": "Ship it", "done": true }),
    )]);
    let worth_proof::TransitionOutcome::Success(json_state) = json_state else {
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
