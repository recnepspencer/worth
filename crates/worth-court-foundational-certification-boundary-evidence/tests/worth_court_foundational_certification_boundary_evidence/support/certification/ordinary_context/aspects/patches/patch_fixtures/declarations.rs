pub(crate) use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::AbsenceLaw, worth_foundational::AspectContract, worth_foundational::AspectValue, worth_foundational::FieldDeclaration, worth_foundational::FieldRequirement, worth_foundational::ScalarAspectType, worth_foundational::StructAspectShape, worth_foundational::StructAspectValue};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision};

pub(crate) fn admitted_state<const N: usize>(
    entries: [worth_foundational::ContractValidatedAspectArtifact; N],
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let worth_proof::TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(entries) else {
        panic!("expected authoritative state admission");
    };
    state
}

pub(crate) fn task_summary_contract() -> AspectContract {
    let shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            field("done"),
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            field("note"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Optional,
            worth_foundational::AbsenceLaw::Optional,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    worth_foundational::AspectContract::struct_aspect(key("task.summary"), identity(20), revision(1), shape)
}

pub(crate) fn validated_task_summary(
    contract: &AspectContract,
    title: &str,
    done: bool,
    note: Option<&str>,
) -> worth_foundational::ContractValidatedAspectArtifact {
    let mut fields = vec![
        (field("title"), worth_foundational::AspectValue::String(title.into())),
        (field("done"), worth_foundational::AspectValue::Bool(done)),
    ];
    if let Some(note) = note {
        fields.push((field("note"), worth_foundational::AspectValue::String(note.into())));
    }
    let value = worth_foundational::StructAspectValue::new(fields).expect("unique fields");
    let worth_proof::TransitionOutcome::Success(artifact) =
        worth_foundational::validate_aspect_value(contract, value.into())
    else {
        panic!("expected validated task summary");
    };
    artifact
}