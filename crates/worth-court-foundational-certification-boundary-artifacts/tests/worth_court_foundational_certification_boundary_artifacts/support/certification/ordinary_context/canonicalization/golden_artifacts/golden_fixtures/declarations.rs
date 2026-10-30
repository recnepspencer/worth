pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::prepare_aspect_contract_for_digest,
    worth_foundational::prepare_aspect_mask_for_digest,
    worth_foundational::prepare_aspect_patch_for_digest,
    worth_foundational::prepare_aspect_state_for_digest, worth_foundational::AspectContract,
    worth_foundational::AspectMask, worth_foundational::AspectValue,
    worth_foundational::CanonicalDigestAspectShapeKind,
    worth_foundational::CanonicalDigestMaskMode,
    worth_foundational::CanonicalDigestPreparationEntry,
    worth_foundational::DigestPreparationMaskMode, worth_foundational::FieldDeclaration,
    worth_foundational::FieldRequirement, worth_foundational::ScalarAspectType,
    worth_foundational::StructAspectShape,
};

pub(crate) use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

pub(crate) fn admitted_state<const N: usize>(
    entries: [worth_foundational::ContractValidatedAspectArtifact; N],
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let worth_proof::TransitionOutcome::Success(state) =
        worth_foundational::admit_authoritative_record_aspect_state(entries)
    else {
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

pub(crate) fn ready_patch(
    patch: &worth_foundational::AuthoritativeRecordAspectPatch,
) -> worth_foundational::DigestPreparationReadyAspectPatchArtifact {
    match prepare_aspect_patch_for_digest(patch) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

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
    Mode: DigestPreparationMaskMode,
{
    match prepare_aspect_mask_for_digest(aspect_key, mask) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    }
}

pub(crate) fn golden_contract() -> AspectContract {
    let shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent title field"),
        worth_foundational::FieldDeclaration::new(
            field("done"),
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent done field"),
    ])
    .expect("unique fields");

    worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(20),
        revision(1),
        shape,
    )
}

pub(crate) fn golden_contract_basis() -> Vec<CanonicalDigestPreparationEntry> {
    vec![
        worth_foundational::CanonicalDigestPreparationEntry::ContractHeader {
            key: key("task.summary"),
            identity: identity(20),
            revision: revision(1),
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractShape {
            key: key("task.summary"),
            shape: worth_foundational::CanonicalDigestAspectShapeKind::Struct,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractStructField {
            key: key("task.summary"),
            field: field("done"),
            value_type: worth_foundational::ScalarAspectType::Bool,
            requirement: worth_foundational::FieldRequirement::Required,
            absence: worth_foundational::AbsenceLaw::Required,
            evolution: worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractStructField {
            key: key("task.summary"),
            field: field("title"),
            value_type: worth_foundational::ScalarAspectType::String,
            requirement: worth_foundational::FieldRequirement::Required,
            absence: worth_foundational::AbsenceLaw::Required,
            evolution: worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractMaskMode {
            key: key("task.summary"),
            mode: worth_foundational::CanonicalDigestMaskMode::Projection,
            allowed: true,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractMaskMode {
            key: key("task.summary"),
            mode: worth_foundational::CanonicalDigestMaskMode::Mutation,
            allowed: true,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractMaskMode {
            key: key("task.summary"),
            mode: worth_foundational::CanonicalDigestMaskMode::Diagnostic,
            allowed: true,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractAbsenceLaw {
            key: key("task.summary"),
            absence: worth_foundational::AbsenceLaw::Required,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractEquivalenceBasis {
            key: key("task.summary"),
            equivalence: worth_foundational::AspectEquivalenceBasis::DeclaredStructFields,
        },
        worth_foundational::CanonicalDigestPreparationEntry::ContractEvolutionPolicy {
            key: key("task.summary"),
            evolution: worth_foundational::AspectEvolutionPolicy::AdditiveFieldsAllowed,
        },
    ]
}

pub(crate) fn golden_mutation_mask_basis() -> Vec<CanonicalDigestPreparationEntry> {
    vec![
        worth_foundational::CanonicalDigestPreparationEntry::MaskFieldPath {
            key: key("task.summary"),
            mode: worth_foundational::CanonicalDigestMaskMode::Mutation,
            path: worth_foundational::CanonicalFieldPath::single(field("done")),
        },
        worth_foundational::CanonicalDigestPreparationEntry::MaskFieldPath {
            key: key("task.summary"),
            mode: worth_foundational::CanonicalDigestMaskMode::Mutation,
            path: worth_foundational::CanonicalFieldPath::single(field("title")),
        },
    ]
}

pub(crate) fn golden_patch_basis() -> Vec<CanonicalDigestPreparationEntry> {
    vec![
        worth_foundational::CanonicalDigestPreparationEntry::PatchWholeAspectSet {
            key: key("task.summary"),
            revision: revision(1),
        },
        worth_foundational::CanonicalDigestPreparationEntry::PatchWholeAspectStructFieldValue {
            key: key("task.summary"),
            field: field("done"),
            value: worth_foundational::AspectValue::Bool(true),
        },
        worth_foundational::CanonicalDigestPreparationEntry::PatchWholeAspectStructFieldValue {
            key: key("task.summary"),
            field: field("title"),
            value: worth_foundational::AspectValue::String("Ship it".into()),
        },
    ]
}
