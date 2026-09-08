use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};

#[test]
fn only_exact_indeterminate_wal_projection_targets_carry_allocation_gap_provenance() {
    let proof = admitted(65, 65, RecoveryOperationFate::Indeterminate).unwrap();
    let targets = proof.observation_targets();
    assert_eq!(targets.len(), 1);
    assert!(proof.contains_exact_observation_target(&targets[0]));
    assert_eq!(
        targets[0].identity(),
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 65,
            generation: 1
        }
    );

    // Equally well-formed materialization in another member is still not this
    // admitted member's exact scope; changing only a target cannot forge closure.
    let foreign = admitted(66, 66, RecoveryOperationFate::Indeterminate).unwrap();
    assert!(!proof.contains_exact_observation_target(&foreign.observation_targets()[0]));
    assert_eq!(
        admitted(66, 65, RecoveryOperationFate::Indeterminate),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    let materialized = admitted(65, 65, RecoveryOperationFate::AcknowledgedDurable).unwrap();
    assert!(!materialized.contains_exact_observation_target(&targets[0]));
}

fn admitted(
    target_page: u64,
    materialized_page: u64,
    fate: RecoveryOperationFate,
) -> Result<AdmittedPhysicalRedoMembers, PhysicalRedoPlanningDenial> {
    let mut target = canonical_target_bytes_with_generations(1, 1);
    target[9..17].copy_from_slice(&target_page.to_le_bytes());
    let projection = projection_with_page_allocation(1, 1, 1, 1, 1, materialized_page);
    let encoded = encoded_redo_with_projection_and_digest(
        &target,
        projection,
        Sha256::digest(result_bytes_for_page(1, materialized_page)).into(),
    );
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    admit_physical_redo_members(
        vec![PhysicalRedoMemberInput::new(
            range(),
            [1; 32],
            fate,
            &encoded,
        )],
        test_store(),
        format,
        PhysicalRedoAdmissionLimits {
            targets: 4,
            distinct_targets: 4,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: 4,
                record_identities: 4,
                placements: 4,
                segment_updates: 4,
                manifests: 4,
                total_entries: 12,
                inline_allocations: 4,
            },
        },
    )
}
