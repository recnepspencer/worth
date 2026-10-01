//! Shape and selected-checkpoint binding for the bounded tag-7 custody roster.
//! Store's attestation is not a semantic release proof; the caller must still
//! join each batch to selected routes, C9 WAL membership and durable fate.

use worth_store_physical_format::{
    decode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
    CheckpointCertificateKind, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1,
};
use worth_store_recovery_physics::PhysicalCheckpointBase;

#[cfg(test)]
#[path = "certificates/no_release_tests.rs"]
mod no_release_tests;

pub(super) struct SelectedReleaseRoster {
    batches: Vec<ReleaseCheckpointBatchV1>,
    accumulator: ReleaseCheckpointAccumulatorV1,
}

impl SelectedReleaseRoster {
    pub(super) fn batches(&self) -> &[ReleaseCheckpointBatchV1] {
        &self.batches
    }

    pub(super) const fn accumulator(&self) -> ReleaseCheckpointAccumulatorV1 {
        self.accumulator
    }
}

pub(super) fn selected_roster(
    selected: &PhysicalCheckpointBase,
) -> Result<Option<SelectedReleaseRoster>, ()> {
    let checkpoint = selected.checkpoint();
    parse_records(
        checkpoint.certificate_records(),
        checkpoint.source().identity(),
        checkpoint.source().root().generation(),
        selected.source_root_frame_sha256(),
    )
}

fn parse_records(
    records: &[Box<[u8]>],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    generation: u64,
    root_sha256: [u8; 32],
) -> Result<Option<SelectedReleaseRoster>, ()> {
    let mut batches = Vec::new();
    let mut accumulator = None;
    let mut no_release = false;
    let mut release_started = false;
    for frame in records {
        let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| ())?;
        match kind {
            CheckpointCertificateKind::TierEpoch if !release_started => {}
            CheckpointCertificateKind::ReleasedDrop => {
                release_started = true;
                match ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| ())? {
                    ReleaseCheckpointCertificateV1::Batch(batch)
                        if !no_release
                            && accumulator.is_none()
                            && batch.checkpoint() == identity
                            && batch.root_generation() == generation
                            && batch.root_sha256() == root_sha256
                            && usize::from(batch.ordinal()) == batches.len() =>
                    {
                        batches.push(batch);
                    }
                    ReleaseCheckpointCertificateV1::Accumulator(value)
                        if !no_release
                            && accumulator.is_none()
                            && value.checkpoint() == identity
                            && value.root_generation() == generation
                            && value.root_sha256() == root_sha256 =>
                    {
                        accumulator = Some(value);
                    }
                    ReleaseCheckpointCertificateV1::NoRelease(value)
                        if !no_release
                            && accumulator.is_none()
                            && batches.is_empty()
                            && value.checkpoint() == identity
                            && value.root_generation() == generation
                            && value.root_sha256() == root_sha256 =>
                    {
                        no_release = true;
                    }
                    _ => return Err(()),
                }
            }
            _ => return Err(()),
        }
    }
    if no_release {
        return Ok(None);
    }
    let Some(accumulator) = accumulator else {
        return if batches.is_empty() {
            Ok(None)
        } else {
            Err(())
        };
    };
    if usize::from(accumulator.batch_count()) != batches.len() {
        return Err(());
    }
    if batches.is_empty() {
        if accumulator.batch_records_digest() != [0; 32]
            || accumulator.prior_checkpoint_sequence() == 0
            || accumulator.cumulative_dropped() != accumulator.prior_cumulative_dropped()
            || accumulator.cumulative_digest() != accumulator.prior_cumulative_digest()
        {
            return Err(());
        }
    } else {
        let digest = release_checkpoint_batch_records_digest_v1(&batches).map_err(|_| ())?;
        let tip = *batches.last().ok_or(())?;
        if accumulator.batch_records_digest() != digest
            || Some(accumulator.tip()) != tip.tip_provenance().ok()
            || accumulator.cumulative_dropped() != tip.cumulative_dropped()
            || accumulator.cumulative_digest() != tip.cumulative_digest()
            || accumulator.terminal() != tip.terminal()
            || batches[0].cumulative_dropped() <= accumulator.prior_cumulative_dropped()
        {
            return Err(());
        }
    }
    Ok(Some(SelectedReleaseRoster {
        batches,
        accumulator,
    }))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use sha2::{Digest, Sha256};

    use worth_store_physical_format::{
        encode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
        store_namespace::{
            ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
        },
        CheckpointCertificateKind, OriginalDropReservationRequestV1, PersistedRecordIdentity,
        PhysicalCheckpointIdentity, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointBatchV1,
        ReleaseCheckpointCertificateV1, ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1,
    };

    use super::parse_records;

    #[test]
    fn selected_roster_binds_exact_checkpoint_root_and_ordered_batch_digest() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
        )
        .published_identity();
        let checkpoint = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap());
        let record = |ordinal| PersistedRecordIdentity::new([2; 16], ordinal).unwrap();
        let batch = ReleaseCheckpointBatchV1::new(
            checkpoint,
            12,
            [3; 32],
            0,
            record(1),
            [4; 32],
            [5; 32],
            record(2),
            [6; 32],
            OriginalDropReservationRequestV1::new([7; 32], [8; 32], 1, 2).unwrap(),
            ReleasedDropWalFateWitnessV1::new(10, 20, [9; 32], [10; 32]).unwrap(),
            11,
            [11; 32],
            None,
            1,
            [12; 32],
            true,
        )
        .unwrap();
        let digest = release_checkpoint_batch_records_digest_v1(&[batch]).unwrap();
        let accumulator = ReleaseCheckpointAccumulatorV1::new(
            checkpoint,
            12,
            [3; 32],
            0,
            [0; 32],
            [0; 32],
            0,
            [0; 32],
            1,
            digest,
            batch.tip_provenance().unwrap(),
            1,
            [12; 32],
            true,
        )
        .unwrap();
        let records = [
            encode_checkpoint_certificate(
                CheckpointCertificateKind::ReleasedDrop,
                &ReleaseCheckpointCertificateV1::Batch(batch).encode(),
            )
            .unwrap()
            .into_boxed_slice(),
            encode_checkpoint_certificate(
                CheckpointCertificateKind::ReleasedDrop,
                &ReleaseCheckpointCertificateV1::Accumulator(accumulator).encode(),
            )
            .unwrap()
            .into_boxed_slice(),
        ];
        assert_eq!(
            parse_records(&records, checkpoint, 12, [3; 32])
                .unwrap()
                .unwrap()
                .batches()
                .len(),
            1
        );
        assert!(parse_records(&records, checkpoint, 12, [99; 32]).is_err());
        assert!(parse_records(&records[..1], checkpoint, 12, [3; 32]).is_err());
        let altered_first_batch = ReleaseCheckpointBatchV1::new(
            checkpoint,
            12,
            [3; 32],
            0,
            batch.descriptor_record(),
            [42; 32],
            batch.custody_digest(),
            batch.reservation_record(),
            batch.reservation_frame_sha256(),
            batch.request(),
            batch.fate(),
            batch.candidate_root_generation(),
            batch.candidate_root_sha256(),
            batch.predecessor(),
            batch.cumulative_dropped(),
            batch.cumulative_digest(),
            batch.terminal(),
        )
        .unwrap();
        let altered_first_batch_frame = encode_checkpoint_certificate(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::Batch(altered_first_batch).encode(),
        )
        .unwrap()
        .into_boxed_slice();
        assert!(
            parse_records(
                &[altered_first_batch_frame, records[1].clone()],
                checkpoint,
                12,
                [3; 32],
            )
            .is_err(),
            "a substituted first Batch must not inherit the selected accumulator"
        );
        assert!(parse_records(
            &[records[1].clone(), records[0].clone()],
            checkpoint,
            12,
            [3; 32]
        )
        .is_err());
        assert!(
            parse_records(
                &[records[0].clone(), records[1].clone(), records[1].clone()],
                checkpoint,
                12,
                [3; 32],
            )
            .is_err(),
            "duplicate selected accumulator cannot mint custody"
        );
        let disputed_terminal = ReleaseCheckpointAccumulatorV1::new(
            checkpoint,
            12,
            [3; 32],
            0,
            [0; 32],
            [0; 32],
            0,
            [0; 32],
            1,
            digest,
            batch.tip_provenance().unwrap(),
            1,
            [12; 32],
            false,
        )
        .unwrap();
        let disputed_terminal_frame = encode_checkpoint_certificate(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::Accumulator(disputed_terminal).encode(),
        )
        .unwrap()
        .into_boxed_slice();
        assert!(
            parse_records(
                &[records[0].clone(), disputed_terminal_frame],
                checkpoint,
                12,
                [3; 32]
            )
            .is_err(),
            "terminal disagreement must deny even with matching tip and digest"
        );
        let forged_tip = ReleasedDropTipProvenanceV1::new(
            batch.descriptor_record(),
            batch.descriptor_frame_sha256(),
            batch.reservation_record(),
            batch.reservation_frame_sha256(),
            batch.request(),
            ReleasedDropWalFateWitnessV1::new(10, 20, [9; 32], [24; 32]).unwrap(),
            batch.candidate_root_generation(),
            batch.candidate_root_sha256(),
        )
        .unwrap();
        let forged_fate = ReleaseCheckpointAccumulatorV1::new(
            checkpoint, 12, [3; 32], 0, [0; 32], [0; 32], 0, [0; 32], 1, digest, forged_tip, 1,
            [12; 32], true,
        )
        .unwrap();
        let forged_fate_frame = encode_checkpoint_certificate(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::Accumulator(forged_fate).encode(),
        )
        .unwrap()
        .into_boxed_slice();
        assert!(
            parse_records(
                &[records[0].clone(), forged_fate_frame],
                checkpoint,
                12,
                [3; 32]
            )
            .is_err(),
            "accumulator fate must match its selected Batch tip"
        );
        let successor = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(4).unwrap());
        let carried = ReleaseCheckpointAccumulatorV1::new(
            successor,
            13,
            [13; 32],
            checkpoint.sequence().get(),
            [3; 32],
            Sha256::digest(accumulator.encode()).into(),
            1,
            [12; 32],
            0,
            [0; 32],
            batch.tip_provenance().unwrap(),
            1,
            [12; 32],
            true,
        )
        .unwrap();
        let carried_frame = encode_checkpoint_certificate(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::Accumulator(carried).encode(),
        )
        .unwrap()
        .into_boxed_slice();
        let carried_roster = parse_records(&[carried_frame], successor, 13, [13; 32])
            .unwrap()
            .unwrap();
        assert!(carried_roster.batches().is_empty());
        assert_eq!(
            carried_roster.accumulator().tip(),
            batch.tip_provenance().unwrap()
        );
    }
}
