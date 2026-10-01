//! One canonical released-drop accumulator step. Store and C8 derive these
//! bytes from independently selected facts; the digest is not itself proof.

use sha2::{Digest, Sha256};

use crate::{PersistedRecordIdentity, ReleasedDropPredecessorV1, MAXIMUM_DROP_SET_RECORDS};

use super::{
    ReleaseCheckpointBatchV1, ReleaseCheckpointCertificateDenial, ReleasedDropWalFateWitnessV1,
};

const DOMAIN: &[u8] = b"store.physical.checkpoint.released-drop-step.v1";
const BATCH_RECORDS_DOMAIN: &[u8] = b"store.physical.checkpoint.released-drop-batches.v1";

/// Ordered tag-7 Batch payloads for one selected checkpoint, including their
/// checkpoint/root wrappers. C8 recomputes this over the exact decoded
/// selected certificate order, not over a caller-provided summary.
pub fn release_checkpoint_batch_records_digest_v1(
    batches: &[ReleaseCheckpointBatchV1],
) -> Result<[u8; 32], ReleaseCheckpointCertificateDenial> {
    let Some(first) = batches.first().copied() else {
        return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
    };
    if batches.len() > 63 {
        return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
    }
    let mut digest = Sha256::new();
    digest.update((BATCH_RECORDS_DOMAIN.len() as u64).to_le_bytes());
    digest.update(BATCH_RECORDS_DOMAIN);
    digest.update((batches.len() as u16).to_le_bytes());
    for (index, batch) in batches.iter().copied().enumerate() {
        if usize::from(batch.ordinal()) != index
            || batch.checkpoint() != first.checkpoint()
            || batch.root_generation() != first.root_generation()
            || batch.root_sha256() != first.root_sha256()
            || batches[..index].iter().any(|prior| {
                prior.descriptor_record() == batch.descriptor_record()
                    || prior.reservation_record() == batch.reservation_record()
            })
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        if let Some(prior) = index.checked_sub(1).map(|index| batches[index]) {
            if batch.cumulative_dropped() <= prior.cumulative_dropped() {
                return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
            }
        }
        let encoded = batch.encode();
        digest.update((encoded.len() as u32).to_le_bytes());
        digest.update(encoded);
    }
    Ok(digest.finalize().into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedDropCumulativeEvidenceV1 {
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    custody_digest: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    fate: ReleasedDropWalFateWitnessV1,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
    predecessor: Option<ReleasedDropPredecessorV1>,
    batch_dropped: u16,
    terminal: bool,
}

impl ReleasedDropCumulativeEvidenceV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
        custody_digest: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        fate: ReleasedDropWalFateWitnessV1,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
        predecessor: Option<ReleasedDropPredecessorV1>,
        batch_dropped: u16,
        terminal: bool,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        if descriptor_record == reservation_record
            || [
                descriptor_frame_sha256,
                custody_digest,
                reservation_frame_sha256,
                candidate_root_sha256,
            ]
            .contains(&[0; 32])
            || candidate_root_generation == 0
            || batch_dropped == 0
            || usize::from(batch_dropped) > MAXIMUM_DROP_SET_RECORDS
            || predecessor.is_some_and(|prior| prior.descriptor_record() == descriptor_record)
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            descriptor_record,
            descriptor_frame_sha256,
            custody_digest,
            reservation_record,
            reservation_frame_sha256,
            fate,
            candidate_root_generation,
            candidate_root_sha256,
            predecessor,
            batch_dropped,
            terminal,
        })
    }

    /// The zero digest is legal only at trusted fresh genesis. A later caller
    /// must supply the selected prior accumulator digest and matching count.
    pub fn advance(
        self,
        prior_dropped: u64,
        prior_digest: [u8; 32],
    ) -> Result<(u64, [u8; 32]), ReleaseCheckpointCertificateDenial> {
        if (prior_dropped == 0) != (prior_digest == [0; 32]) {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        let cumulative = prior_dropped
            .checked_add(u64::from(self.batch_dropped))
            .ok_or(ReleaseCheckpointCertificateDenial::InvalidBinding)?;
        let mut digest = Sha256::new();
        digest.update((DOMAIN.len() as u64).to_le_bytes());
        digest.update(DOMAIN);
        digest.update(prior_dropped.to_le_bytes());
        digest.update(prior_digest);
        write_record(&mut digest, self.descriptor_record);
        digest.update(self.descriptor_frame_sha256);
        digest.update(self.custody_digest);
        write_record(&mut digest, self.reservation_record);
        digest.update(self.reservation_frame_sha256);
        digest.update(self.fate.lsn_start().to_le_bytes());
        digest.update(self.fate.lsn_end_exclusive().to_le_bytes());
        digest.update(self.fate.identity_digest());
        digest.update(self.fate.payload_digest());
        digest.update(self.candidate_root_generation.to_le_bytes());
        digest.update(self.candidate_root_sha256);
        if let Some(prior) = self.predecessor {
            digest.update([1]);
            write_record(&mut digest, prior.descriptor_record());
            digest.update(prior.descriptor_frame_sha256());
        } else {
            digest.update([0]);
        }
        digest.update(self.batch_dropped.to_le_bytes());
        digest.update([u8::from(self.terminal)]);
        Ok((cumulative, digest.finalize().into()))
    }
}

fn write_record(digest: &mut Sha256, record: PersistedRecordIdentity) {
    digest.update(record.allocation_epoch());
    digest.update(record.ordinal().to_le_bytes());
}
