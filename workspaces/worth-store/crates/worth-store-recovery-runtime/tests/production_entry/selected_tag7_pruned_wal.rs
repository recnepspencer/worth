//! A native cutover may retain tag-7 custody after its old C.9 WAL is gone.

use std::{fs, num::NonZeroU64, path::Path};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, RecordAppendBatch,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn native_cutover_prunes_old_released_wal_but_selected_tag7_reopens() {
    // The native policy is large enough for every real V3 frame, but bounded
    // so a later checkpoint can retire its fully covered old segment.
    let world =
        super::pending_wal_world::first_with_wal_segment_bytes(NonZeroU64::new(1 << 20).unwrap());
    let outcome =
        WorthStoreRecovery::recover(super::certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("first killed V3 must recover before native cutover: {outcome:?}")
    };
    let seal = handoff.into_core().into_checkpoint_custody().unwrap();
    let serving = super::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        world.root(),
        seal,
        NonZeroU64::new(1 << 20).unwrap(),
    );
    serving
        .retire_displaced_segment()
        .expect("native retirement must publish the first Batch checkpoint");
    let c1 = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (first_batches, first_accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&c1);
    assert_eq!(first_batches.len(), 1);
    assert_eq!(
        first_accumulator.base().tip(),
        first_batches[0].tip_provenance().unwrap()
    );
    let first_heads = super::release_reopen::selected_head_oracle::selected_heads(
        world.root(),
        first_accumulator,
    );

    // Only a sealed prefix can be reclaimed. Complete enough ordinary
    // publications to rotate the 1 MiB WAL segment containing the old V3;
    // unresolved durability-only padding would keep that prefix live.
    rotate_completed_wal(&serving);

    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x9d; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("idle accumulator-only checkpoint must admit after native cutover");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let c2 = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(c2, c1);
    let (idle_batches, idle_accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&c2);
    assert!(idle_batches.is_empty());
    assert_eq!(
        idle_accumulator.base().tip(),
        first_accumulator.base().tip()
    );
    assert_eq!(
        idle_accumulator.prior_head_count(),
        first_accumulator.head_count()
    );
    assert_eq!(
        idle_accumulator.prior_head_roster_digest(),
        first_accumulator.head_roster_digest()
    );
    let heads =
        super::release_reopen::selected_head_oracle::selected_heads(world.root(), idle_accumulator);
    assert_eq!(heads, first_heads);
    let witness = idle_accumulator.base().tip().fate();
    assert!(
        !retained_wal_interval(
            world.root(),
            witness.lsn_start(),
            witness.lsn_end_exclusive(),
        ),
        "native cutover must physically prune the old V3 WAL frame",
    );

    let outcome =
        WorthStoreRecovery::recover(super::certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("selected tag-7 without old WAL must reopen: {outcome:?}")
    };
    let seal = handoff.into_core().into_checkpoint_custody().unwrap();
    super::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        world.root(),
        seal,
        NonZeroU64::new(1 << 20).unwrap(),
    )
    .close();
}

pub(super) fn rotate_completed_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(format))
        .unwrap();
    let submission = serving.record_submission();
    for ordinal in 0..24_u8 {
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
                [0xe0 + ordinal; 32],
            ))
            .unwrap();
        let payload = vec![ordinal; 64 * 1024];
        let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
            submission
                .prepare_durable_append(
                    RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
                    placement,
                    PhysicalMutationRequest::platform_durable(
                        key,
                        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                    ),
                )
                .into_raw()
        else {
            panic!("completed ordinary WAL rotation {ordinal} must prepare")
        };
        assert!(matches!(
            prepared.execute(),
            PhysicalMutationOutcome::Completed(_)
        ));
    }
}

pub(super) fn retained_wal_interval(root: &Path, start: u64, end: u64) -> bool {
    const HEADER: usize = 116;
    const FOOTER: usize = 32;
    for entry in fs::read_dir(root.join("families/wal")).unwrap() {
        let bytes = fs::read(entry.unwrap().path()).unwrap();
        let mut offset = 0_usize;
        while offset < bytes.len() {
            let header = bytes
                .get(offset..offset + HEADER)
                .expect("complete WAL header");
            assert_eq!(&header[..8], b"WORTHWAL");
            let frame_start = u64::from_le_bytes(header[28..36].try_into().unwrap());
            let frame_end = u64::from_le_bytes(header[36..44].try_into().unwrap());
            let payload =
                usize::try_from(u64::from_le_bytes(header[44..52].try_into().unwrap())).unwrap();
            let next = offset
                .checked_add(HEADER)
                .and_then(|value| value.checked_add(payload))
                .and_then(|value| value.checked_add(FOOTER))
                .expect("bounded WAL frame extent");
            assert!(next <= bytes.len() && frame_start < frame_end);
            if frame_start == start && frame_end == end {
                return true;
            }
            offset = next;
        }
    }
    false
}
