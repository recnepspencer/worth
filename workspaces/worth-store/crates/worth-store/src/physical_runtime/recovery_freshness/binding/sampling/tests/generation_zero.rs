//! The generation-zero basis opens only under the sampled coordination's own
//! absence witness, and stays closed to reclaim and maintenance frames.

use super::super::super::{wal_frame_input::RecoveryWalFrameInput, CheckpointCoveredMembers};
use super::*;
use crate::physical_runtime::{
    AbsentCheckpointWitness, CheckpointMemoryLimit, ConfiguredPhysicalDurabilityDeclaration,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy,
    PhysicalDurabilityDeclaration, PhysicalWalPolicy, RetainedWalTailLimit,
    StoreRecoveryBindingSampleDenial as Denial, StoreRecoverySamplingBasis, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};
use std::num::NonZeroU32;
use worth_store_wal::{LogSequenceNumber, WalLsnRange, WAL_ORIGIN};

struct Frame {
    range: WalLsnRange,
    payload: Vec<u8>,
}

impl RecoveryWalFrameInput for Frame {
    fn recovery_lsn_range(&self) -> WalLsnRange {
        self.range
    }

    fn recovery_payload(&self) -> &[u8] {
        &self.payload
    }
}

fn frame(payload: Vec<u8>) -> Frame {
    let start = WAL_ORIGIN.lsn().get();
    Frame {
        range: WalLsnRange::new(
            LogSequenceNumber::new(start),
            LogSequenceNumber::new(start + 1),
        )
        .unwrap(),
        payload,
    }
}

fn declaration() -> ConfiguredPhysicalDurabilityDeclaration {
    PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(2).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(16).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(4096).unwrap()),
        ))
}

/// Installs the observed absence and returns the media with its witness.
fn absent(
    coordination: &mut PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
) -> (AdmittedRecoveryFilesystemMedia, AbsentCheckpointWitness) {
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let observed =
        crate::physical_runtime::recovery_coordination::observe_checkpoint_for_test(&mut discovery)
            .unwrap();
    assert!(observed.bytes().is_none());
    let media = discovery.finish();
    let witness = coordination.install_absent_checkpoint(observed).unwrap();
    (media, witness)
}

fn sample(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    witness: &AbsentCheckpointWitness,
    frames: &[Frame],
) -> Result<(), Denial> {
    super::super::sample_binding_from_frames(
        CheckpointCoveredMembers::Skip,
        coordination,
        media,
        StoreRecoverySamplingBasis::GenerationZero(witness, declaration()),
        frames.iter(),
        16,
        4096,
        4096,
    )
    .map(drop)
    .map_err(|failure| failure.denial())
}

fn cleanup_intent(store: [u8; 16]) -> Vec<u8> {
    BlobManifestResidueCleanupV1::intent(
        store,
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [4; 32],
        [5; 32],
        [6; 32],
        [7; 32],
        1,
        2,
        [8; 32],
        16,
        1,
    )
    .unwrap()
    .encode()
}

#[test]
fn generation_zero_samples_only_under_its_own_coordinations_absence_witness() {
    with_owner(|coordination, media, _| {
        let (media, witness) = absent(coordination, media);
        assert_eq!(sample(coordination, &media, &witness, &[]), Ok(()));
        with_owner(|other, other_media, _| {
            // The other coordination also observed absence; only its own
            // witness opens its basis.
            let (other_media, own) = absent(other, other_media);
            assert_eq!(sample(other, &other_media, &own, &[]), Ok(()));
            assert_eq!(
                sample(other, &other_media, &witness, &[]),
                Err(Denial::GenerationZeroWithoutAbsentCheckpoint)
            );
        });
    });
}

#[test]
fn generation_zero_denies_extent_copy_and_residue_cleanup_frames_before_effect() {
    with_owner(|coordination, media, _| {
        let (media, witness) = absent(coordination, media);
        let mut copy = EXTENT_COPY_DOMAIN.to_vec();
        copy.extend_from_slice(&[0; 8]);
        assert_eq!(
            sample(coordination, &media, &witness, &[frame(copy)]),
            Err(Denial::GenerationZeroExtentCopy)
        );
        let cleanup = cleanup_intent(media.store_identity().bytes());
        assert!(BlobManifestResidueCleanup::decode(&cleanup).is_ok());
        assert_eq!(
            sample(coordination, &media, &witness, &[frame(cleanup)]),
            Err(Denial::GenerationZeroResidueCleanup)
        );
    });
}
