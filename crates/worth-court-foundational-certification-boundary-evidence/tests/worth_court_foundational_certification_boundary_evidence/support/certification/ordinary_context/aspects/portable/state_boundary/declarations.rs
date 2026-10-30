pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::export_portable_record_aspect_state,
    worth_foundational::readmit_portable_record_aspect_state, worth_foundational::AspectContract,
    worth_foundational::AspectValue, worth_foundational::ContractValidationInput,
    worth_foundational::PortableAspectContractBasis,
    worth_foundational::PortableAspectReadmissionDenial,
    worth_foundational::PortableRecordAspectState,
    worth_foundational::PortableRecordAspectStateEntry, worth_foundational::ScalarAspectType,
};

pub(crate) use {
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::admitted_state,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::validated_task_summary,
};
pub(crate) use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

pub(crate) fn lookup(
    contract: &worth_foundational::AspectContract,
) -> impl Fn(&worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> + '_ {
    move |key| (key == contract.key()).then(|| contract.clone())
}
