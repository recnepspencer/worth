//! Reconcile the descriptor's exact Store idempotency with authenticated C8
//! operation evidence. A missing operation is never a no-effect witness.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobManifestResidueCleanup, OriginalDropProofV1};
use worth_store_recovery_physics::{ReconciledOperationFates, RecoveryOperationFate};

use crate::orchestration::planning::resolved_basis::ResolvedPlanningBasis;

const DROP_MATERIAL_DOMAIN: &[u8] = b"worth.store.blob.reclaim.mutation.v1";
const IDEMPOTENCY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

pub(super) fn proves_descriptor_no_effect(
    intent: BlobManifestResidueCleanup,
    basis: &ResolvedPlanningBasis,
) -> bool {
    let OriginalDropProofV1::ProvenNoEffect { idempotency, .. } = intent.proof() else {
        return false;
    };
    matching_original_drop_evidence(
        intent,
        &basis.fates,
        basis.sample.policy_identity(),
        basis
            .sample
            .wal_members()
            .iter()
            .any(|member| member.operation_identity() == idempotency),
    )
}

fn matching_original_drop_evidence(
    intent: impl Into<BlobManifestResidueCleanup>,
    fates: &ReconciledOperationFates,
    policy: [u8; 32],
    has_wal_member: bool,
) -> bool {
    let intent = intent.into();
    let OriginalDropProofV1::ProvenNoEffect {
        idempotency,
        fingerprint,
        ..
    } = intent.proof()
    else {
        return false;
    };
    let mut matching = fates
        .operations()
        .iter()
        .filter(|operation| operation.identity().idempotency() == idempotency);
    let Some(operation) = matching.next() else {
        return false;
    };
    if matching.next().is_some() {
        return false;
    }
    operation.identity().store() == intent.store()
        && operation.fate() == RecoveryOperationFate::ProvenNoEffect
        && operation.request_fingerprint() == fingerprint
        && expected_drop_key(
            intent.store(),
            intent.reclaim_attempt(),
            policy,
            operation.lease_issuance_generation(),
            operation.lease_expiry_generation(),
        ) == idempotency
        && !has_wal_member
}

pub(super) fn proves_never_reserved(
    intent: BlobManifestResidueCleanup,
    basis: &ResolvedPlanningBasis,
) -> bool {
    if !matches!(intent, BlobManifestResidueCleanup::V2(_))
        || intent.proof() != OriginalDropProofV1::NeverReserved
    {
        return false;
    }
    // Any authenticated original-drop reservation/binding contradicts the
    // selected NeverReserved slot; absence alone is never this proof.
    let policy = basis.sample.policy_identity();
    !basis.fates.operations().iter().any(|operation| {
        operation.identity().store() == intent.store()
            && expected_drop_key(
                intent.store(),
                intent.reclaim_attempt(),
                policy,
                operation.lease_issuance_generation(),
                operation.lease_expiry_generation(),
            ) == operation.identity().idempotency()
    }) && !basis.redo.projections().iter().any(|projection| {
        let Some(bytes) = basis
            .redo
            .blob_semantic_record_bytes(projection.operation())
        else {
            return false;
        };
        matches!(worth_store_physical_format::decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(value))
                    if value.store() == intent.store()
                        && value.reclaim_attempt() == intent.reclaim_attempt())
    })
}

/// Tag 3 is an explicit recovered-registry certificate, not an inference from
/// an absent binding. C8 independently checks the complete checkpoint/tail
/// operation, WAL-member, and redo evidence for contradictory drop material.
pub(super) fn proves_recovered_no_binding(
    intent: BlobManifestResidueCleanup,
    basis: &ResolvedPlanningBasis,
) -> bool {
    let OriginalDropProofV1::RecoveredNoBinding { idempotency, .. } = intent.proof() else {
        return false;
    };
    if !matches!(intent, BlobManifestResidueCleanup::V2(_)) {
        return false;
    }
    let policy = basis.sample.policy_identity();
    let conflicting_operation = basis
        .fates
        .operations()
        .iter()
        .any(|operation| is_recovered_drop_material(intent, policy, idempotency, operation));
    let conflicting_member = basis.sample.wal_members().iter().any(|member| {
        member.operation_identity() == idempotency
            || basis.fates.operations().iter().any(|operation| {
                operation.identity().idempotency() == member.operation_identity()
                    && is_recovered_drop_material(intent, policy, idempotency, operation)
            })
    });
    let conflicting_redo = basis.redo.projections().iter().any(|projection| {
        let Some(bytes) = basis
            .redo
            .blob_semantic_record_bytes(projection.operation())
        else {
            return false;
        };
        conflicting_recovered_redo_descriptor(intent, bytes)
    });
    !conflicting_operation && !conflicting_member && !conflicting_redo
}

fn conflicting_recovered_redo_descriptor(intent: BlobManifestResidueCleanup, bytes: &[u8]) -> bool {
    matches!(worth_store_physical_format::decode_blob_record(bytes),
        Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(value))
            if value.store() == intent.store()
                && (value.reclaim_attempt() == intent.reclaim_attempt()
                    || value.manifest_record() == intent.manifest_record()))
}

fn is_recovered_drop_material(
    intent: BlobManifestResidueCleanup,
    policy: [u8; 32],
    exact_key: [u8; 32],
    operation: &worth_store_recovery_physics::ReconciledOperationFate,
) -> bool {
    operation.identity().idempotency() == exact_key
        || (operation.identity().store() == intent.store()
            && expected_drop_key(
                intent.store(),
                intent.reclaim_attempt(),
                policy,
                operation.lease_issuance_generation(),
                operation.lease_expiry_generation(),
            ) == operation.identity().idempotency())
}

pub(super) fn recovered_reservation_key_matches(
    intent: BlobManifestResidueCleanup,
    policy: [u8; 32],
    reservation: worth_store_physical_format::OriginalDropReservedV1,
) -> bool {
    let OriginalDropProofV1::RecoveredNoBinding {
        idempotency,
        fingerprint,
        ..
    } = intent.proof()
    else {
        return false;
    };
    reservation.request().idempotency() == idempotency
        && reservation.request().fingerprint() == fingerprint
        && expected_drop_key(
            intent.store(),
            intent.reclaim_attempt(),
            policy,
            reservation.request().lease_issuance_generation(),
            reservation.request().lease_expiry_generation(),
        ) == idempotency
}

pub(in crate::orchestration::planning::completion::blob_reclaim) fn expected_drop_key(
    store: [u8; 16],
    attempt: [u8; 16],
    policy: [u8; 32],
    issuance: u64,
    expiry: u64,
) -> [u8; 32] {
    let mut material = Sha256::new();
    material.update(DROP_MATERIAL_DOMAIN);
    material.update(store);
    material.update(attempt);
    material.update([2]);
    let mut key = Sha256::new();
    key.update((IDEMPOTENCY_DOMAIN.len() as u64).to_le_bytes());
    key.update(IDEMPOTENCY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material.finalize());
    key.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{BlobManifestResidueCleanupV1, PersistedRecordIdentity};
    use worth_store_recovery_physics::{
        reconcile_operation_fates, RecoveryBindingFreshness, RecoveryOperationEvidenceInput,
        RecoveryOperationIdentity,
    };

    fn intent() -> BlobManifestResidueCleanupV1 {
        let store = [1; 16];
        let attempt = [2; 16];
        BlobManifestResidueCleanupV1::intent(
            store,
            attempt,
            PersistedRecordIdentity::new([3; 16], 4).unwrap(),
            [5; 32],
            [6; 32],
            expected_drop_key(store, attempt, [7; 32], 1, 10),
            [8; 32],
            9,
            10,
            [11; 32],
            12,
            13,
        )
        .unwrap()
    }

    fn fates(
        intent: BlobManifestResidueCleanupV1,
        fingerprint: [u8; 32],
        fate: RecoveryOperationFate,
    ) -> ReconciledOperationFates {
        let identity = RecoveryOperationIdentity::new(
            intent.store(),
            1,
            1,
            1,
            intent.drop_idempotency_identity(),
        )
        .unwrap();
        reconcile_operation_fates(
            2,
            vec![RecoveryOperationEvidenceInput::new(
                identity,
                fingerprint,
                1,
                10,
                RecoveryBindingFreshness::Retained,
                fate,
            )],
            1,
        )
        .unwrap()
    }

    #[test]
    fn only_exact_authenticated_no_effect_without_descriptor_member_admits_cleanup() {
        let intent = intent();
        let proven = fates(
            intent,
            intent.drop_request_fingerprint(),
            RecoveryOperationFate::ProvenNoEffect,
        );
        assert!(matching_original_drop_evidence(
            intent, &proven, [7; 32], false
        ));
        assert!(!matching_original_drop_evidence(
            intent, &proven, [7; 32], true
        ));
        assert!(!matching_original_drop_evidence(
            intent, &proven, [9; 32], false
        ));
        let pending = fates(
            intent,
            intent.drop_request_fingerprint(),
            RecoveryOperationFate::Indeterminate,
        );
        assert!(!matching_original_drop_evidence(
            intent, &pending, [7; 32], false
        ));
        let wrong_fingerprint = fates(intent, [9; 32], RecoveryOperationFate::ProvenNoEffect);
        assert!(!matching_original_drop_evidence(
            intent,
            &wrong_fingerprint,
            [7; 32],
            false,
        ));
        let absent = reconcile_operation_fates(2, vec![], 1).unwrap();
        assert!(!matching_original_drop_evidence(
            intent, &absent, [7; 32], false
        ));
    }

    #[test]
    fn descriptor_key_binds_store_attempt_policy_and_lease() {
        let base = expected_drop_key([1; 16], [2; 16], [3; 32], 4, 5);
        assert_ne!(base, expected_drop_key([9; 16], [2; 16], [3; 32], 4, 5));
        assert_ne!(base, expected_drop_key([1; 16], [9; 16], [3; 32], 4, 5));
        assert_ne!(base, expected_drop_key([1; 16], [2; 16], [9; 32], 4, 5));
        assert_ne!(base, expected_drop_key([1; 16], [2; 16], [3; 32], 9, 5));
        assert_ne!(base, expected_drop_key([1; 16], [2; 16], [3; 32], 4, 9));
    }
}

#[cfg(test)]
#[path = "wal_fate/recovered_tests.rs"]
mod recovered_tests;
