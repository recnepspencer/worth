pub(crate) use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::prepare_aspect_contract_for_digest, worth_foundational::prepare_aspect_mask_for_digest, worth_foundational::prepare_aspect_patch_for_digest, worth_foundational::prepare_aspect_state_for_digest, worth_foundational::AbsenceLaw, worth_foundational::AspectContract, worth_foundational::AspectMask, worth_foundational::FieldDeclaration, worth_foundational::FieldRequirement, worth_foundational::ScalarAspectType, worth_foundational::StructAspectShape};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision};

pub(crate) fn ready_contract(
    contract: AspectContract,
) -> worth_foundational::DigestPreparationReadyAspectContractArtifact {
    match prepare_aspect_contract_for_digest(contract) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

pub(crate) fn ready_mask<Mode>(
    aspect_key: worth_foundational::AspectKey,
    mask: AspectMask<Mode>,
) -> worth_foundational::DigestPreparationReadyAspectMaskArtifact<Mode>
where
    Mode: worth_foundational::DigestPreparationMaskMode,
{
    match prepare_aspect_mask_for_digest(aspect_key, mask) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

pub(crate) fn ready_patch(
    patch: &worth_foundational::AuthoritativeRecordAspectPatch,
) -> worth_foundational::DigestPreparationReadyAspectPatchArtifact {
    match prepare_aspect_patch_for_digest(patch) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

pub(crate) fn admitted_state<const N: usize>(
    entries: [worth_foundational::ContractValidatedAspectArtifact; N],
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let worth_proof::TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(entries) else {
        panic!("expected authoritative state admission");
    };
    state
}

pub(crate) fn ready_state(
    state: worth_foundational::AuthoritativeRecordAspectStateArtifact,
) -> worth_foundational::DigestPreparationReadyAspectStateArtifact {
    match prepare_aspect_state_for_digest(state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

pub(crate) fn task_summary_contract() -> AspectContract {
    worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(20),
        revision(1),
        task_summary_shape([
            ("title", worth_foundational::ScalarAspectType::String),
            ("done", worth_foundational::ScalarAspectType::Bool),
        ]),
    )
}

pub(crate) fn task_summary_contract_with_reversed_declaration_order() -> AspectContract {
    worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(20),
        revision(1),
        task_summary_shape([
            ("done", worth_foundational::ScalarAspectType::Bool),
            ("title", worth_foundational::ScalarAspectType::String),
        ]),
    )
}

pub(crate) fn task_summary_shape<const N: usize>(fields: [(&str, ScalarAspectType); N]) -> StructAspectShape {
    worth_foundational::StructAspectShape::new(fields.map(|(name, value_type)| {
        worth_foundational::FieldDeclaration::new(
            field(name),
            value_type,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law")
    }))
    .expect("unique fields")
}