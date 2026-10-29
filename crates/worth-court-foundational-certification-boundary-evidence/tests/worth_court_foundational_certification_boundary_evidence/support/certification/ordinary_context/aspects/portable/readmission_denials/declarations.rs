pub(crate) use {worth_foundational::readmit_portable_record_aspect_patch, worth_foundational::AspectContract, worth_foundational::AspectValue, worth_foundational::CanonicalTime, worth_foundational::ContractValidationDenial, worth_foundational::ContractValidationInput, worth_foundational::PortableAspectContractBasis, worth_foundational::PortableAspectFieldSet, worth_foundational::PortableAspectPatchOperation, worth_foundational::PortableAspectReadmissionDenial, worth_foundational::PortablePatchReadmissionPurpose, worth_foundational::PortableRecordAspectPatch, worth_foundational::ScalarAspectType};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract};
pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision};

pub(crate) fn lookup(
    contract: &worth_foundational::AspectContract,
) -> impl Fn(&worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> + '_ {
    move |key| (key == contract.key()).then(|| contract.clone())
}

pub(crate) fn missing_lookup(_: &worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> {
    None
}