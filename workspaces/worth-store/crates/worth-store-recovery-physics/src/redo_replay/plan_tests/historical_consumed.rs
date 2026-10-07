use super::*;
use crate::HistoricalReleasedDropTargetWitness;
use worth_store_physical_format::RecordFrameCoordinate;

const OLD: [u8; 32] = [0x31; 32];
const DESCRIPTOR: [u8; 32] = [0x72; 32];
const ROOT: [u8; 32] = [0x83; 32];

fn admitted_target() -> (AdmittedPhysicalRedoMembers, PhysicalRedoTarget) {
    let admitted = admit_physical_redo_members(
        vec![PhysicalRedoMemberInput::new(
            range(),
            OLD,
            RecoveryOperationFate::Indeterminate,
            &encoded_redo(),
        )],
        test_store(),
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: u64::MAX,
            targets: 2,
            distinct_targets: 2,
            projection: worth_store_physical_format::PhysicalRecoveryProjectionDecodeLimits {
                frames: 2,
                record_identities: 2,
                placements: 2,
                segment_updates: 2,
                manifests: 2,
                total_entries: 6,
                inline_allocations: 2,
            },
        },
    )
    .unwrap();
    let target = admitted.observation_targets()[0].clone();
    (admitted, target)
}

fn witness(target: &PhysicalRedoTarget) -> HistoricalReleasedDropTargetWitness {
    HistoricalReleasedDropTargetWitness {
        selected_root_identity: ROOT,
        descriptor_operation: DESCRIPTOR,
        old_operation: OLD,
        target: target.identity(),
        wal_target_digest: target.resulting_digest(),
        target_coordinate: RecordFrameCoordinate::new(
            target.artifact(),
            target.artifact_offset(),
            target.artifact_length(),
        )
        .unwrap(),
    }
}

#[test]
fn complete_exact_old_group_is_consumed_only_after_verified_descriptor() {
    let (admitted, target) = admitted_target();
    let observation =
        RecoveryPageObservation::historical_released_drop(&target, witness(&target)).unwrap();
    let mut plan = admitted.plan(vec![observation]).unwrap();
    assert_eq!(plan.counters().skip_historical_drop(), 1);
    assert_eq!(plan.counters().apply(), 0);
    assert!(plan
        .admit_historical_consumed_operations(&[], ROOT)
        .is_none());
    assert!(plan
        .admit_historical_consumed_operations(&[DESCRIPTOR], [0; 32])
        .is_none());
    let consumed = plan
        .admit_historical_consumed_operations(&[DESCRIPTOR], ROOT)
        .unwrap();
    assert_eq!(consumed.operations().collect::<Vec<_>>(), vec![OLD]);
    assert_eq!(consumed.descriptor_for(OLD), Some(DESCRIPTOR));

    plan.decisions[0].kind = PhysicalRedoDecisionKind::Apply;
    assert!(plan
        .admit_historical_consumed_operations(&[DESCRIPTOR], ROOT)
        .is_none());
}

#[test]
fn swapped_target_operation_coordinate_and_digest_cannot_be_consumed() {
    let (admitted, target) = admitted_target();
    let mut swapped = witness(&target);
    swapped.target = PhysicalRedoTargetIdentity::InlinePage {
        segment: 2,
        page: 2,
        generation: 2,
    };
    assert!(RecoveryPageObservation::historical_released_drop(&target, swapped).is_none());

    let mut wrong_digest = witness(&target);
    wrong_digest.wal_target_digest[0] ^= 1;
    assert!(RecoveryPageObservation::historical_released_drop(&target, wrong_digest).is_none());

    let mut wrong_coordinate = witness(&target);
    wrong_coordinate.target_coordinate = RecordFrameCoordinate::new(
        target.artifact(),
        target.artifact_offset() + 1,
        target.artifact_length(),
    )
    .unwrap();
    assert!(RecoveryPageObservation::historical_released_drop(&target, wrong_coordinate).is_none());

    let mut wrong_operation = witness(&target);
    wrong_operation.old_operation = [0x44; 32];
    let observation =
        RecoveryPageObservation::historical_released_drop(&target, wrong_operation).unwrap();
    assert!(admitted.plan(vec![observation]).is_err());
}
