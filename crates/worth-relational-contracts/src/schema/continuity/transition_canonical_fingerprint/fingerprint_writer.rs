use sha2::{Digest, Sha256};

use crate::schema::data::{SchemaBoundaryFingerprint, SchemaDiffAtom};

use super::detail_ordering::detail_sort_key;
use super::normalized_transition::{
    normalize_transition, CanonicalSchemaDiffAtom, CanonicalSchemaDiffDetail,
};

pub fn fingerprint_transition(diff_atoms: &[SchemaDiffAtom]) -> SchemaBoundaryFingerprint {
    let normalized_transition = normalize_transition(diff_atoms);
    let mut hasher = Sha256::new();
    for atom in &normalized_transition.canonical_atoms {
        write_atom_to_hasher(&mut hasher, atom);
    }

    let digest: [u8; 32] = hasher.finalize().into();
    SchemaBoundaryFingerprint::new(digest)
}

fn write_atom_to_hasher(hasher: &mut Sha256, atom: &CanonicalSchemaDiffAtom<'_>) {
    update_tagged_bytes(hasher, atom.atom.element.schema_id.0.as_bytes());
    hasher.update(atom.atom.element.schema_version_id.0.to_le_bytes());
    hasher.update([atom.atom.element.kind as u8]);
    match atom.atom.element.kind_id {
        Some(kind_id) => {
            hasher.update([1]);
            hasher.update(kind_id.0.to_le_bytes());
        }
        None => hasher.update([0]),
    }
    update_tagged_bytes(hasher, atom.atom.element.element_name.as_bytes());
    hasher.update((atom.normalized_strata.len() as u64).to_le_bytes());
    for stratum in &atom.normalized_strata {
        hasher.update([*stratum as u8]);
    }
    hasher.update([atom.atom.publication_impact as u8]);
    hasher.update([atom.atom.subscriber_impact as u8]);
    hasher.update([atom.atom.historical_interpretation as u8]);
    write_detail_to_hasher(hasher, &atom.normalized_detail);
}

fn write_detail_to_hasher(hasher: &mut Sha256, detail: &CanonicalSchemaDiffDetail<'_>) {
    hasher.update([detail_sort_key(detail)]);
    match detail {
        CanonicalSchemaDiffDetail::AddedField {
            field,
            required,
            default_expression,
        } => {
            update_tagged_bytes(hasher, field.as_str().as_bytes());
            hasher.update([u8::from(*required)]);
            match default_expression {
                Some(expr) => {
                    hasher.update([1]);
                    update_tagged_bytes(hasher, expr.as_bytes());
                }
                None => hasher.update([0]),
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
            hasher.update((added_variants.len() as u64).to_le_bytes());
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
            hasher.update([*declared_intent as u8]);
        }
    }
}

fn update_tagged_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
