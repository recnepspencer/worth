use sha2::{Digest, Sha256};
use worth_foundational::canonicalization_api::lower_lane::basis::CanonicalizationRuleVersion;
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily, StoreDigestEquivalenceBasis,
    StoreEquivalenceBasisIdentity, StorePhysicalMutationRequestCanonicalFields,
    StorePhysicalMutationRequestCanonicalSource,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::request::PhysicalMutationDurabilityRequest;
use crate::physical_runtime::PhysicalDurabilityPolicyIdentity;

const DOMAIN: &str = "store.physical.mutation.request-fingerprint.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMutationRequestFingerprint {
    digest: [u8; 32],
    basis: StoreEquivalenceBasisIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationOperationFamily {
    RecordAppend,
    SegmentRewrite,
    ExtentRewrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationRequestScope {
    family: u8,
    identity: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationPayloadDigest([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationSecurityBasis([u8; 32]);

pub(in crate::physical_runtime) struct PhysicalMutationFingerprintInput<'a> {
    pub store: StableStoreIdentity,
    pub durability_policy: PhysicalDurabilityPolicyIdentity,
    pub scope: PhysicalMutationRequestScope,
    pub payload: PhysicalMutationPayloadDigest,
    pub durability_request: PhysicalMutationDurabilityRequest,
    pub operation_family: PhysicalMutationOperationFamily,
    pub security_bases: &'a [PhysicalMutationSecurityBasis],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationFingerprintDenial {
    InvalidCanonicalSource,
    CanonicalAdmissionDenied,
}

pub(super) fn reopen_exact_native_fingerprint(
    digest: [u8; 32],
) -> PhysicalMutationRequestFingerprint {
    PhysicalMutationRequestFingerprint {
        digest,
        basis: StoreDigestEquivalenceBasis::exact_native_basis(
            StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint,
        )
        .identity(),
    }
}

impl PhysicalMutationRequestFingerprint {
    pub(in crate::physical_runtime) fn derive(
        input: PhysicalMutationFingerprintInput<'_>,
    ) -> Result<Self, PhysicalMutationFingerprintDenial> {
        let mut security_bases = input
            .security_bases
            .iter()
            .map(|basis| basis.0)
            .collect::<Vec<_>>();
        security_bases.sort_unstable();
        let canonical_fields = StorePhysicalMutationRequestCanonicalFields {
            store: input.store.bytes(),
            durability_policy: input.durability_policy.bytes(),
            scope_family: input.scope.family,
            scope_identity: input.scope.identity,
            payload: input.payload.0,
            durability_request: durability_code(input.durability_request),
            operation_family: operation_code(input.operation_family),
        };
        let source = StorePhysicalMutationRequestCanonicalSource::new(
            canonical_fields,
            security_bases.iter().copied(),
        )
        .map_err(|_| PhysicalMutationFingerprintDenial::InvalidCanonicalSource)?;
        let version = CanonicalizationRuleVersion::new(DOMAIN)
            .ok_or(PhysicalMutationFingerprintDenial::CanonicalAdmissionDenied)?;
        match StoreCanonicalBasisConstruction::for_family(
            StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint,
        )
        .with_physical_mutation_request(&source)
        .prepare(version)
        {
            TransitionOutcome::Success(_) => {}
            _ => return Err(PhysicalMutationFingerprintDenial::CanonicalAdmissionDenied),
        }
        let digest = fingerprint_bytes(canonical_fields, &security_bases);
        Ok(Self {
            digest,
            basis: StoreDigestEquivalenceBasis::exact_native_basis(
                StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint,
            )
            .identity(),
        })
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.digest
    }

    pub const fn equivalence_basis(self) -> StoreEquivalenceBasisIdentity {
        self.basis
    }
}

impl PhysicalMutationRequestScope {
    pub(in crate::physical_runtime) const fn record_append(identity: [u8; 32]) -> Self {
        Self {
            family: 1,
            identity,
        }
    }
}

impl PhysicalMutationPayloadDigest {
    pub(in crate::physical_runtime) const fn from_validated_payload(digest: [u8; 32]) -> Self {
        Self(digest)
    }
}

impl PhysicalMutationSecurityBasis {
    pub(in crate::physical_runtime) const fn from_admitted_security(identity: [u8; 32]) -> Self {
        Self(identity)
    }
}

fn fingerprint_bytes(
    fields: StorePhysicalMutationRequestCanonicalFields,
    security_bases: &[[u8; 32]],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    write_field(&mut digest, DOMAIN.as_bytes());
    write_field(&mut digest, &fields.store);
    write_field(&mut digest, &fields.durability_policy);
    write_field(&mut digest, &[fields.scope_family]);
    write_field(&mut digest, &fields.scope_identity);
    write_field(&mut digest, &fields.payload);
    write_field(&mut digest, &[fields.durability_request]);
    write_field(&mut digest, &[fields.operation_family]);
    write_field(&mut digest, &(security_bases.len() as u32).to_le_bytes());
    for basis in security_bases {
        write_field(&mut digest, basis);
    }
    digest.finalize().into()
}

fn write_field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

const fn durability_code(request: PhysicalMutationDurabilityRequest) -> u8 {
    match request {
        PhysicalMutationDurabilityRequest::PlatformDurable => 1,
    }
}

const fn operation_code(family: PhysicalMutationOperationFamily) -> u8 {
    match family {
        PhysicalMutationOperationFamily::RecordAppend => 1,
        PhysicalMutationOperationFamily::SegmentRewrite => 2,
        PhysicalMutationOperationFamily::ExtentRewrite => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_golden_vector_changes_every_effect_relevant_field() {
        let fields = StorePhysicalMutationRequestCanonicalFields {
            store: [1; 16],
            durability_policy: [2; 32],
            scope_family: 1,
            scope_identity: [3; 32],
            payload: [4; 32],
            durability_request: 1,
            operation_family: 1,
        };
        let base = fingerprint_bytes(fields, &[[5; 32]]);
        assert_eq!(
            hex(&base),
            "150043ebf5c10d8d0fba65839c8cfde0772e6d842b7536766cc02848afe42b62"
        );
        for changed_fields in [
            StorePhysicalMutationRequestCanonicalFields {
                store: [9; 16],
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                durability_policy: [9; 32],
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                scope_family: 9,
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                scope_identity: [9; 32],
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                payload: [9; 32],
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                durability_request: 9,
                ..fields
            },
            StorePhysicalMutationRequestCanonicalFields {
                operation_family: 9,
                ..fields
            },
        ] {
            assert_ne!(base, fingerprint_bytes(changed_fields, &[[5; 32]]));
        }
        assert_ne!(base, fingerprint_bytes(fields, &[[9; 32]]));
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
