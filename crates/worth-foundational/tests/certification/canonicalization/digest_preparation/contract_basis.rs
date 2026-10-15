use worth_foundational::{
    aspect_contract_digest_preparation_basis, AbsenceLaw, CanonicalDigestAspectShapeKind,
    CanonicalDigestMaskMode, CanonicalDigestPreparationEntry, FieldRequirement, ScalarAspectType,
};

use super::readiness_fixtures::{
    ready_contract, task_summary_contract, task_summary_contract_with_reversed_declaration_order,
};
use crate::certification::foundational_vocabulary::{field, identity, key, revision};

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
            CanonicalDigestPreparationEntry::ContractHeader {
                key: key("task.summary"),
                identity: identity(20),
                revision: revision(1),
            },
            CanonicalDigestPreparationEntry::ContractShape {
                key: key("task.summary"),
                shape: CanonicalDigestAspectShapeKind::Struct,
            },
            CanonicalDigestPreparationEntry::ContractStructField {
                key: key("task.summary"),
                field: field("done"),
                value_type: ScalarAspectType::Bool,
                requirement: FieldRequirement::Required,
                absence: AbsenceLaw::Required,
                evolution: worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
            },
            CanonicalDigestPreparationEntry::ContractStructField {
                key: key("task.summary"),
                field: field("title"),
                value_type: ScalarAspectType::String,
                requirement: FieldRequirement::Required,
                absence: AbsenceLaw::Required,
                evolution: worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
            },
            CanonicalDigestPreparationEntry::ContractMaskMode {
                key: key("task.summary"),
                mode: CanonicalDigestMaskMode::Projection,
                allowed: true,
            },
            CanonicalDigestPreparationEntry::ContractMaskMode {
                key: key("task.summary"),
                mode: CanonicalDigestMaskMode::Mutation,
                allowed: true,
            },
            CanonicalDigestPreparationEntry::ContractMaskMode {
                key: key("task.summary"),
                mode: CanonicalDigestMaskMode::Diagnostic,
                allowed: true,
            },
            CanonicalDigestPreparationEntry::ContractAbsenceLaw {
                key: key("task.summary"),
                absence: AbsenceLaw::Required,
            },
            CanonicalDigestPreparationEntry::ContractEquivalenceBasis {
                key: key("task.summary"),
                equivalence: worth_foundational::AspectEquivalenceBasis::DeclaredStructFields,
            },
            CanonicalDigestPreparationEntry::ContractEvolutionPolicy {
                key: key("task.summary"),
                evolution: worth_foundational::AspectEvolutionPolicy::AdditiveFieldsAllowed,
            },
        ]
    );
}
