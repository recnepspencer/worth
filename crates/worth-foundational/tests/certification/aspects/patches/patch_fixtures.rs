use worth_foundational::{
    admit_authoritative_record_aspect_state, AbsenceLaw, AspectContract, AspectValue,
    FieldDeclaration, FieldRequirement, ScalarAspectType, StructAspectShape, StructAspectValue,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::{field, identity, key, revision};

pub(crate) fn admitted_state<const N: usize>(
    entries: [worth_foundational::ContractValidatedAspectArtifact; N],
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(entries) else {
        panic!("expected authoritative state admission");
    };
    state
}

pub(crate) fn task_summary_contract() -> AspectContract {
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            field("title"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            field("done"),
            ScalarAspectType::Bool,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            field("note"),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    AspectContract::struct_aspect(key("task.summary"), identity(20), revision(1), shape)
}

pub(crate) fn validated_task_summary(
    contract: &AspectContract,
    title: &str,
    done: bool,
    note: Option<&str>,
) -> worth_foundational::ContractValidatedAspectArtifact {
    let mut fields = vec![
        (field("title"), AspectValue::String(title.into())),
        (field("done"), AspectValue::Bool(done)),
    ];
    if let Some(note) = note {
        fields.push((field("note"), AspectValue::String(note.into())));
    }
    let value = StructAspectValue::new(fields).expect("unique fields");
    let TransitionOutcome::Success(artifact) =
        worth_foundational::validate_aspect_value(contract, value.into())
    else {
        panic!("expected validated task summary");
    };
    artifact
}
