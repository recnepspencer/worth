use worth_proof::TransitionOutcome;

use super::digest_entries::digest_basis_for_aspect_contract;
use super::semantic_tokens::{
    absence_law_name, aspect_equivalence_basis_name, aspect_evolution_policy_name,
    digest_mask_mode_name, digest_shape_name, field_requirement_name, opaque_aspect_type_name,
    reference_aspect_type_name, scalar_aspect_type_name,
};
use crate::aspects::AspectContract;
use crate::canonicalization::{
    prepare_canonical_basis_sequence, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisValue, CanonicalDigestPreparationEntry, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};

pub fn prepare_aspect_contract_for_canonical_basis(
    version: CanonicalizationRuleVersion,
    contract: AspectContract,
) -> TransitionOutcome<CanonicalBasisReadyArtifact, CanonicalBasisConstructionDenial> {
    prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::AspectContract,
        canonical_basis_for_aspect_contract(&contract),
    )
}

fn canonical_basis_for_aspect_contract(contract: &AspectContract) -> Vec<CanonicalBasisEntry> {
    digest_basis_for_aspect_contract(contract)
        .into_iter()
        .flat_map(canonical_entries_for_contract_digest_entry)
        .collect()
}

fn canonical_entries_for_contract_digest_entry(
    entry: CanonicalDigestPreparationEntry,
) -> Vec<CanonicalBasisEntry> {
    match entry {
        CanonicalDigestPreparationEntry::ContractHeader {
            key,
            identity,
            revision,
        } => vec![
            contract_entry(
                key.as_str(),
                "header.identity",
                CanonicalBasisEntryKind::Header,
                unsigned_64(identity.0),
            ),
            contract_entry(
                key.as_str(),
                "header.revision",
                CanonicalBasisEntryKind::Header,
                unsigned_64(revision.0),
            ),
        ],
        CanonicalDigestPreparationEntry::ContractShape { key, shape } => vec![contract_entry(
            key.as_str(),
            "shape",
            CanonicalBasisEntryKind::Shape,
            exact_text(digest_shape_name(shape)),
        )],
        CanonicalDigestPreparationEntry::ContractScalarShape { key, scalar } => {
            vec![contract_entry(
                key.as_str(),
                "shape.scalar",
                CanonicalBasisEntryKind::Shape,
                exact_text(scalar_aspect_type_name(scalar)),
            )]
        }
        CanonicalDigestPreparationEntry::ContractOpaqueShape { key, opaque } => {
            vec![contract_entry(
                key.as_str(),
                "shape.opaque",
                CanonicalBasisEntryKind::Shape,
                exact_text(opaque_aspect_type_name(opaque)),
            )]
        }
        CanonicalDigestPreparationEntry::ContractReferenceShape { key, reference } => {
            vec![contract_entry(
                key.as_str(),
                "shape.reference",
                CanonicalBasisEntryKind::Shape,
                exact_text(reference_aspect_type_name(reference)),
            )]
        }
        CanonicalDigestPreparationEntry::ContractStructField {
            key,
            field,
            value_type,
            requirement,
            absence,
            evolution,
        } => vec![
            contract_entry(
                key.as_str(),
                format!("field.{}.value_type", field.as_str()),
                CanonicalBasisEntryKind::Field,
                exact_text(scalar_aspect_type_name(value_type)),
            ),
            contract_entry(
                key.as_str(),
                format!("field.{}.requirement", field.as_str()),
                CanonicalBasisEntryKind::Field,
                exact_text(field_requirement_name(requirement)),
            ),
            contract_entry(
                key.as_str(),
                format!("field.{}.absence", field.as_str()),
                CanonicalBasisEntryKind::Field,
                exact_text(absence_law_name(absence)),
            ),
            contract_entry(
                key.as_str(),
                format!("field.{}.evolution", field.as_str()),
                CanonicalBasisEntryKind::Field,
                exact_text(aspect_evolution_policy_name(evolution)),
            ),
        ],
        CanonicalDigestPreparationEntry::ContractFieldCodec { key, field, codec } => {
            vec![contract_entry(
                key.as_str(),
                format!("field.{}.codec", field.as_str()),
                CanonicalBasisEntryKind::Field,
                CanonicalBasisValue::BytesDigest(codec),
            )]
        }
        CanonicalDigestPreparationEntry::ContractMaskMode { key, mode, allowed } => {
            vec![contract_entry(
                key.as_str(),
                format!("mask.{}", digest_mask_mode_name(mode)),
                CanonicalBasisEntryKind::Mask,
                CanonicalBasisValue::Bool(allowed),
            )]
        }
        CanonicalDigestPreparationEntry::ContractAbsenceLaw { key, absence } => {
            vec![contract_entry(
                key.as_str(),
                "absence",
                CanonicalBasisEntryKind::Header,
                exact_text(absence_law_name(absence)),
            )]
        }
        CanonicalDigestPreparationEntry::ContractEquivalenceBasis { key, equivalence } => {
            vec![contract_entry(
                key.as_str(),
                "equivalence",
                CanonicalBasisEntryKind::Header,
                exact_text(aspect_equivalence_basis_name(equivalence)),
            )]
        }
        CanonicalDigestPreparationEntry::ContractEvolutionPolicy { key, evolution } => {
            vec![contract_entry(
                key.as_str(),
                "evolution",
                CanonicalBasisEntryKind::Header,
                exact_text(aspect_evolution_policy_name(evolution)),
            )]
        }
        _ => unreachable!("contract canonical basis only consumes contract digest entries"),
    }
}

fn contract_entry(
    aspect_key: &str,
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::AspectContract,
        CanonicalBasisLocus::Named(format!("{aspect_key}.{}", locus.into()).into()),
        kind,
        value,
    )
}

fn exact_text(value: &'static str) -> CanonicalBasisValue {
    CanonicalBasisValue::ExactText(value.into())
}

fn unsigned_64(value: u64) -> CanonicalBasisValue {
    CanonicalBasisValue::UnsignedInteger {
        width: CanonicalIntegerWidth::Bits64,
        value: u128::from(value),
    }
}
