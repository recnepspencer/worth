use crate::aspect_wire::encoding_sink::{CanonicalAspectOutput, CanonicalHasher};

use crate::schema::data::{SchemaBoundaryFingerprint, SchemaDiffAtom};

use super::detail_ordering::detail_sort_key;
use super::normalized_transition::{
    normalize_transition, CanonicalSchemaDiffAtom, CanonicalSchemaDiffDetail,
};

pub fn fingerprint_transition(diff_atoms: &[SchemaDiffAtom]) -> SchemaBoundaryFingerprint {
    let normalized_transition = normalize_transition(diff_atoms);
    let mut hasher = CanonicalHasher::default();
    for atom in &normalized_transition.canonical_atoms {
        write_atom_to_hasher(&mut hasher, atom);
    }

    SchemaBoundaryFingerprint::new(hasher.finish())
}

fn write_atom_to_hasher(hasher: &mut CanonicalHasher, atom: &CanonicalSchemaDiffAtom<'_>) {
    update_tagged_bytes(hasher, atom.atom.element.schema_id.0.as_bytes());
    hasher
        .u32_le(atom.atom.element.schema_version_id.0)
        .unwrap();
    hasher.byte(atom.atom.element.kind as u8).unwrap();
    match atom.atom.element.kind_id {
        Some(kind_id) => {
            hasher.byte(1).unwrap();
            hasher.u32_le(kind_id.0).unwrap();
        }
        None => hasher.byte(0).unwrap(),
    }
    update_tagged_bytes(hasher, atom.atom.element.element_name.as_bytes());
    hasher.u64_le(atom.normalized_strata.len() as u64).unwrap();
    for stratum in &atom.normalized_strata {
        hasher.byte(*stratum as u8).unwrap();
    }
    hasher.byte(atom.atom.publication_impact as u8).unwrap();
    hasher.byte(atom.atom.subscriber_impact as u8).unwrap();
    hasher
        .byte(atom.atom.historical_interpretation as u8)
        .unwrap();
    write_detail_to_hasher(hasher, &atom.normalized_detail);
}

fn write_detail_to_hasher(hasher: &mut CanonicalHasher, detail: &CanonicalSchemaDiffDetail<'_>) {
    hasher.byte(detail_sort_key(detail)).unwrap();
    match detail {
        CanonicalSchemaDiffDetail::AddedField {
            field,
            required,
            default_expression,
        } => {
            update_tagged_bytes(hasher, field.as_str().as_bytes());
            hasher.byte(u8::from(*required)).unwrap();
            match default_expression {
                Some(expr) => {
                    hasher.byte(1).unwrap();
                    update_tagged_bytes(hasher, expr.as_bytes());
                }
                None => hasher.byte(0).unwrap(),
            }
        }
        CanonicalSchemaDiffDetail::RemovedField { field } => {
            update_tagged_bytes(hasher, field.as_str().as_bytes());
        }
        CanonicalSchemaDiffDetail::TypeChanged {
            field,
            from_type,
            to_type,
        } => {
            update_tagged_bytes(hasher, field.as_str().as_bytes());
            update_tagged_bytes(hasher, from_type.as_bytes());
            update_tagged_bytes(hasher, to_type.as_bytes());
        }
        CanonicalSchemaDiffDetail::EnumDomainExpanded {
            field,
            added_variants,
        } => {
            update_tagged_bytes(hasher, field.as_str().as_bytes());
            hasher.u64_le(added_variants.len() as u64).unwrap();
            for variant in added_variants {
                update_tagged_bytes(hasher, variant.as_bytes());
            }
        }
        CanonicalSchemaDiffDetail::InvariantContractChanged { contract_name }
        | CanonicalSchemaDiffDetail::SubscriberContractChanged { contract_name } => {
            update_tagged_bytes(hasher, contract_name.as_bytes());
        }
        CanonicalSchemaDiffDetail::ProjectionContractChanged { projection_name } => {
            update_tagged_bytes(hasher, projection_name.as_bytes());
        }
        CanonicalSchemaDiffDetail::FreeText {
            detail,
            declared_intent,
        } => {
            update_tagged_bytes(hasher, detail.as_bytes());
            hasher.byte(*declared_intent as u8).unwrap();
        }
    }
}

fn update_tagged_bytes(hasher: &mut CanonicalHasher, bytes: &[u8]) {
    hasher.bytes_with_u64_length(bytes).unwrap();
}
