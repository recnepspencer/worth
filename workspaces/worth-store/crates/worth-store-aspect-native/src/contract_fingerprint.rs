use sha2::{Digest, Sha256};
use worth_foundational::{
    AbsenceLaw, AspectContract, AspectEquivalenceBasis, AspectEvolutionPolicy, AspectMask,
    AspectShape, DiagnosticMask, FieldRequirement, MutationMask, ProjectionMask, ScalarAspectType,
};

use crate::{StoreAspectContractStamp, StorePhysicalBoundaryWitness};

const CONTRACT_FINGERPRINT_DOMAIN: &[u8] = b"worth-store.aspect-native.contract-fingerprint.v1";

pub(crate) fn canonical_contract_fingerprint(contract: &AspectContract) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(CONTRACT_FINGERPRINT_DOMAIN);
    update_text(&mut digest, contract.key().as_str());
    digest.update(contract.identity().0.to_le_bytes());
    digest.update(contract.revision().0.to_le_bytes());
    digest.update([absence_code(contract.absence())]);
    digest.update([equivalence_code(contract.equivalence())]);
    digest.update([evolution_code(contract.evolution())]);
    digest.update([
        u8::from(contract.masks().projection_allowed()),
        u8::from(contract.masks().mutation_allowed()),
        u8::from(contract.masks().diagnostic_allowed()),
    ]);
    update_shape(&mut digest, contract.shape());
    digest.finalize().into()
}

pub(crate) fn canonical_binding_fingerprint(
    contract: StoreAspectContractStamp,
    witness: StorePhysicalBoundaryWitness,
    projection: Option<&AspectMask<ProjectionMask>>,
    mutation: Option<&AspectMask<MutationMask>>,
    diagnostic: Option<&AspectMask<DiagnosticMask>>,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store.aspect-native.signal-binding.v2");
    digest.update(contract.identity().to_le_bytes());
    digest.update(contract.revision().to_le_bytes());
    digest.update(contract.canonical_fingerprint());
    update_physical_witness(&mut digest, witness);
    update_mask(&mut digest, projection);
    update_mask(&mut digest, mutation);
    update_mask(&mut digest, diagnostic);
    digest.finalize().into()
}

fn update_mask<Mode>(digest: &mut Sha256, mask: Option<&AspectMask<Mode>>) {
    let Some(mask) = mask else {
        digest.update([0]);
        return;
    };
    digest.update([1]);
    digest.update((mask.paths().len() as u64).to_le_bytes());
    for path in mask.paths() {
        digest.update((path.fields().len() as u64).to_le_bytes());
        for field in path.fields() {
            update_text(digest, field.as_str());
        }
    }
}

fn update_physical_witness(digest: &mut Sha256, witness: StorePhysicalBoundaryWitness) {
    let authority = witness.authority();
    let scope = authority.roadmap_scope();
    update_text(digest, scope.roadmap());
    update_text(digest, scope.sequence());
    update_text(digest, authority.boundary_instance().label());
    digest.update([match authority.authority_scope() {
        worth_store_contracts::PhysicalAuthorityScope::AspectNativeBoundaryVocabulary => 1,
        worth_store_contracts::PhysicalAuthorityScope::PhysicalFoundationVocabulary => 2,
        worth_store_contracts::PhysicalAuthorityScope::PhysicalEvidenceExport => 3,
        worth_store_contracts::PhysicalAuthorityScope::PhysicalSubstrateReadiness => 4,
    }]);
}

fn update_shape(digest: &mut Sha256, shape: &AspectShape) {
    match shape {
        AspectShape::Scalar(scalar) => digest.update([1, scalar_code(*scalar)]),
        AspectShape::Struct(shape) => {
            digest.update([2]);
            digest.update((shape.fields().len() as u64).to_le_bytes());
            for field in shape.fields() {
                update_text(digest, field.key().as_str());
                digest.update([
                    scalar_code(field.value_type()),
                    requirement_code(field.requirement()),
                    absence_code(field.absence()),
                    evolution_code(field.evolution()),
                ]);
            }
        }
        AspectShape::Opaque(_) => digest.update([3, 1]),
        AspectShape::Reference(_) => digest.update([4, 1]),
        AspectShape::Content => digest.update([5]),
    }
}

fn update_text(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value.as_bytes());
}

const fn absence_code(value: AbsenceLaw) -> u8 {
    match value {
        AbsenceLaw::Required => 1,
        AbsenceLaw::Optional => 2,
        AbsenceLaw::Defaulted => 3,
    }
}

const fn equivalence_code(value: AspectEquivalenceBasis) -> u8 {
    match value {
        AspectEquivalenceBasis::ExactCanonicalValue => 1,
        AspectEquivalenceBasis::DeclaredStructFields => 2,
        AspectEquivalenceBasis::OpaqueIdentity => 3,
        AspectEquivalenceBasis::ReferenceIdentity => 4,
        AspectEquivalenceBasis::ContentIdentity => 5,
    }
}

const fn evolution_code(value: AspectEvolutionPolicy) -> u8 {
    match value {
        AspectEvolutionPolicy::Frozen => 1,
        AspectEvolutionPolicy::AdditiveFieldsAllowed => 2,
        AspectEvolutionPolicy::WideningAllowed => 3,
        AspectEvolutionPolicy::ExplicitBreakRequired => 4,
    }
}

const fn requirement_code(value: FieldRequirement) -> u8 {
    match value {
        FieldRequirement::Required => 1,
        FieldRequirement::Optional => 2,
        FieldRequirement::Defaulted => 3,
    }
}

const fn scalar_code(value: ScalarAspectType) -> u8 {
    match value {
        ScalarAspectType::Null => 1,
        ScalarAspectType::Bool => 2,
        ScalarAspectType::Int8 => 3,
        ScalarAspectType::Int16 => 4,
        ScalarAspectType::Int32 => 5,
        ScalarAspectType::Int64 => 6,
        ScalarAspectType::UInt8 => 7,
        ScalarAspectType::UInt16 => 8,
        ScalarAspectType::UInt32 => 9,
        ScalarAspectType::UInt64 => 10,
        ScalarAspectType::Float32 => 11,
        ScalarAspectType::Float64 => 12,
        ScalarAspectType::Decimal => 13,
        ScalarAspectType::BigInt => 14,
        ScalarAspectType::Rational => 15,
        ScalarAspectType::String => 16,
        ScalarAspectType::Uuid => 18,
        ScalarAspectType::Date => 19,
        ScalarAspectType::Time => 20,
        ScalarAspectType::Timestamp => 21,
        ScalarAspectType::TimestampTz => 22,
        ScalarAspectType::EntityRef => 23,
        ScalarAspectType::ContentRef => 24,
        ScalarAspectType::InlineBytes => 25,
    }
}
