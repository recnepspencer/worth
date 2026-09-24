use sha2::{Digest, Sha256};

use crate::physical_runtime::durability::{
    DecodedPhysicalMutationBindingRecord, PersistedPhysicalMutationAttemptBinding,
    PersistedPhysicalMutationFate,
};
use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalMutationRequestFingerprint};

use super::{
    StoreRecoveryBindingFreshness, StoreRecoveryOperationEvidence, StoreRecoveryOperationFate,
};

pub(super) fn checkpoint_evidence(
    record: DecodedPhysicalMutationBindingRecord,
    selected_generation: u64,
) -> StoreRecoveryOperationEvidence {
    match record {
        DecodedPhysicalMutationBindingRecord::RebuiltUnsealed(basis)
        | DecodedPhysicalMutationBindingRecord::RebuiltGroupSealed { basis, .. } => {
            evidence_from_basis(
                basis.key(),
                basis.fingerprint(),
                basis.mutation(),
                selected_generation,
                StoreRecoveryOperationFate::Indeterminate,
                None,
            )
        }
        DecodedPhysicalMutationBindingRecord::WalBound { persisted, .. } => {
            evidence_from_persisted(
                &persisted,
                selected_generation,
                StoreRecoveryOperationFate::Indeterminate,
            )
        }
        DecodedPhysicalMutationBindingRecord::Terminal { basis, fate } => {
            let fate = match fate {
                PersistedPhysicalMutationFate::Completed(_) => {
                    StoreRecoveryOperationFate::AcknowledgedDurable
                }
                PersistedPhysicalMutationFate::ProvenNoEffect(_) => {
                    StoreRecoveryOperationFate::ProvenNoEffect
                }
                PersistedPhysicalMutationFate::Indeterminate(_) => {
                    StoreRecoveryOperationFate::Indeterminate
                }
            };
            evidence_from_basis(
                basis.key(),
                basis.fingerprint(),
                basis.mutation(),
                selected_generation,
                fate,
                None,
            )
        }
    }
}

pub(super) fn evidence_from_persisted(
    binding: &PersistedPhysicalMutationAttemptBinding,
    selected_generation: u64,
    fate: StoreRecoveryOperationFate,
) -> StoreRecoveryOperationEvidence {
    let attempt_binding_identity = Sha256::digest(binding.bytes()).into();
    evidence_from_basis(
        binding.key(),
        binding.fingerprint(),
        binding.mutation(),
        selected_generation,
        fate,
        Some(attempt_binding_identity),
    )
}

fn evidence_from_basis(
    key: &crate::physical_runtime::PhysicalMutationIdempotencyKey,
    fingerprint: PhysicalMutationRequestFingerprint,
    mutation: PhysicalMutationIdentity,
    selected_generation: u64,
    fate: StoreRecoveryOperationFate,
    attempt_binding_identity: Option<[u8; 32]>,
) -> StoreRecoveryOperationEvidence {
    let lease = key.lease();
    StoreRecoveryOperationEvidence {
        idempotency_identity: key.identity().bytes(),
        mutation,
        request_fingerprint: fingerprint,
        lease_issuance_generation: lease.issuance_generation().get(),
        lease_expiry_generation: lease.expiry_generation().get(),
        freshness: if selected_generation >= lease.expiry_generation().get() {
            StoreRecoveryBindingFreshness::ExpiredAtSelectedCheckpoint
        } else {
            StoreRecoveryBindingFreshness::Retained
        },
        fate,
        attempt_binding_identity,
    }
}
