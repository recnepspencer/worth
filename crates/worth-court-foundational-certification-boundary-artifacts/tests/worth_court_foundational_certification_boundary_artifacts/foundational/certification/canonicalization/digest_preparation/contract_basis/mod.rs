use {
    worth_foundational::aspect_contract_digest_preparation_basis, worth_foundational::AbsenceLaw,
    worth_foundational::CanonicalDigestAspectShapeKind,
    worth_foundational::CanonicalDigestMaskMode,
    worth_foundational::CanonicalDigestPreparationEntry, worth_foundational::FieldRequirement,
    worth_foundational::ScalarAspectType,
};

use {crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::ready_contract, crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::task_summary_contract, crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::task_summary_contract_with_reversed_declaration_order};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision};

#[test]
fn contract_digest_preparation_basis_uses_declared_field_semantics_in_canonical_order() {
    let left_ready = ready_contract(task_summary_contract());
    let right_ready = ready_contract(task_summary_contract_with_reversed_declaration_order());

    assert_eq!(
        aspect_contract_digest_preparation_basis(&left_ready),
        aspect_contract_digest_preparation_basis(&right_ready)
    );
    assert_eq!(
        aspect_contract_digest_preparation_basis(&left_ready),
        &[
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
    );
}
