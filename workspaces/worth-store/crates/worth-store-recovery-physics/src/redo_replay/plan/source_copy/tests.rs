use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::*;
use worth_store_physical_format::*;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

#[test]
fn copy_plan_keeps_two_lsns_and_has_no_synthetic_page_decisions() {
    let (format, member) = fixture(RecoveryOperationFate::Indeterminate);
    let admitted = admit_physical_redo_members(vec![member], store(), format, limits()).unwrap();
    assert!(admitted.observation_targets().is_empty());
    let plan = admitted.plan(vec![]).unwrap();
    assert!(plan.decisions().is_empty());
    assert!(plan.projections().is_empty());
    assert_eq!(plan.source_copies().len(), 1);
    let copy = &plan.source_copies()[0];
    assert_eq!(copy.publication_lsn(), 50);
    assert_eq!(copy.recipe().intent_lsn(), 41);
    assert_eq!(copy.projection().source_root_generation(), 17);
    assert_eq!(plan.recovery_root_allocation_bytes(), 65_536);
    assert_eq!(plan.supersession_scratch_bytes(), 3 * 16_384 + 4096);
}

#[test]
fn copy_rejects_foreign_operation_final_lsn_and_unbounded_memory() {
    let (format, member) = fixture(RecoveryOperationFate::Indeterminate);
    let mut wrong_operation = member.clone();
    wrong_operation.operation = [20; 32];
    assert_eq!(
        admit_physical_redo_members(vec![wrong_operation], store(), format, limits()),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    let mut wrong_lsn = member.clone();
    let offset = 8 + DOMAIN.len();
    wrong_lsn.canonical_redo[offset..offset + 8].copy_from_slice(&41_u64.to_le_bytes());
    assert_eq!(
        admit_physical_redo_members(vec![wrong_lsn], store(), format, limits()),
        Err(PhysicalRedoPlanningDenial::LsnRangeMismatch)
    );
    let mut bounded = limits();
    bounded.recovery_memory_bytes = 3 * 16_384 + 4095;
    assert!(matches!(
        admit_physical_redo_members(vec![member], store(), format, bounded),
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit { .. })
    ));
}

#[test]
fn copy_retains_materialized_fate_and_rejects_false_no_effect() {
    let (format, member) = fixture(RecoveryOperationFate::AcknowledgedDurable);
    let plan = admit_physical_redo_members(vec![member], store(), format, limits())
        .unwrap()
        .plan(vec![])
        .unwrap();
    assert_eq!(
        plan.source_copies()[0].fate(),
        RecoveryOperationFate::AcknowledgedDurable
    );
    assert_eq!(plan.recovery_root_allocation_bytes(), 0);
    let (_, member) = fixture(RecoveryOperationFate::ProvenNoEffect);
    assert_eq!(
        admit_physical_redo_members(vec![member], store(), format, limits()),
        Err(PhysicalRedoPlanningDenial::ProvenNoEffectHasWalAttempt)
    );
}

fn fixture(
    fate: RecoveryOperationFate,
) -> (PhysicalRecordFormatDeclaration, PhysicalRedoMemberInput) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source = DurableExtentRecordPlacement::legacy_unknown(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
        40_000,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
    )
    .unwrap();
    let intent = PhysicalExtentCopyIntent::new(
        format,
        [19; 32],
        12,
        source,
        ExtentArenaRange::new(ExtentArenaId::new(8).unwrap(), 0, 53_248).unwrap(),
        4096,
        [23; 32],
    )
    .unwrap();
    let digest = Sha256::digest(PhysicalExtentCopyRecord::Intent(intent).encode()).into();
    let recipe = PersistedExtentCopyRecipe::new(intent, 41, digest).unwrap();
    let root = PersistedPhysicalRecoveryRootState::new(65_536, 1, 32, vec![], None, None).unwrap();
    let projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, root, recipe).unwrap();
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&(DOMAIN.len() as u64).to_le_bytes());
    encoded.extend_from_slice(DOMAIN);
    encoded.extend_from_slice(&50_u64.to_le_bytes());
    let projection = projection.encode();
    encoded.extend_from_slice(&(projection.len() as u64).to_le_bytes());
    encoded.extend_from_slice(&projection);
    let range = WalLsnRange::new(LogSequenceNumber::new(50), LogSequenceNumber::new(51)).unwrap();
    (
        format,
        PhysicalRedoMemberInput::new(range, [19; 32], fate, &encoded),
    )
}

fn store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity()
}
fn limits() -> PhysicalRedoAdmissionLimits {
    PhysicalRedoAdmissionLimits {
        recovery_memory_bytes: 100_000,
        targets: 0,
        distinct_targets: 0,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 0,
            record_identities: 1,
            placements: 1,
            segment_updates: 0,
            manifests: 0,
            total_entries: 1,
            inline_allocations: 0,
        },
    }
}
