pub(crate) use {worth_foundational::validate_aspect_value, worth_foundational::AbsenceLaw, worth_foundational::AspectContract, worth_foundational::AspectValue, worth_foundational::CanonicalF64, worth_foundational::ContentRefId, worth_foundational::ContractValidatedAspectValueView, worth_foundational::ContractValidationDenial, worth_foundational::EntityId, worth_foundational::FieldDeclaration, worth_foundational::FieldRequirement, worth_foundational::PartitionId, worth_foundational::ScalarAspectType, worth_foundational::StructAspectShape, worth_foundational::StructAspectValue};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision};

pub(crate) fn task_summary_contract() -> AspectContract {
    let shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("done"),
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    worth_foundational::AspectContract::struct_aspect(key("task.summary"), identity(2), revision(1), shape)
}