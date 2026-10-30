pub(crate) use serde_json::json;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::aspect_contract_digest_preparation_basis,
    worth_foundational::aspect_mask_digest_preparation_basis,
    worth_foundational::aspect_patch_digest_preparation_basis,
    worth_foundational::aspect_state_digest_preparation_basis,
    worth_foundational::lower_json_record_aspect_state, worth_foundational::validate_aspect_value,
    worth_foundational::AspectLocator, worth_foundational::AspectMask,
    worth_foundational::AspectValue, worth_foundational::CanonicalDigestPreparationEntry,
    worth_foundational::CanonicalFieldPath, worth_foundational::JsonCompatibilityAspectInput,
    worth_foundational::LocatorAuthority, worth_foundational::MutationMask,
    worth_foundational::StructAspectValue,
};

pub(crate) use {
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::admitted_state,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_contract,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_contract_basis,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_mutation_mask_basis,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::golden_patch_basis,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_contract,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_mask,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_patch,
    crate::certification::ordinary_context::canonicalization::golden_artifacts::golden_fixtures::ready_state,
};
pub(crate) use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
};

#[allow(dead_code)]
pub(crate) fn assert_digest_basis(_basis: &[CanonicalDigestPreparationEntry]) {}
