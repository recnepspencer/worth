//! Fixed V2 cleanup member wire; semantic admission remains on the parent type.

use super::{
    BlobManifestResidueCleanupV2, OriginalDropProofV1, ReservedDropRecordV1,
    BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN, WIRE_BYTES,
};
use crate::PersistedRecordIdentity;

impl BlobManifestResidueCleanupV2 {
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(WIRE_BYTES);
        bytes.extend_from_slice(
            &(BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len() as u64).to_le_bytes(),
        );
        bytes.extend_from_slice(BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN);
        bytes.push(match self.phase {
            crate::BlobManifestResidueCleanupPhaseV1::Intent => 1,
            crate::BlobManifestResidueCleanupPhaseV1::Completed => 2,
        });
        bytes.push(match self.proof {
            OriginalDropProofV1::ProvenNoEffect { .. } => 1,
            OriginalDropProofV1::NeverReserved => 2,
            OriginalDropProofV1::RecoveredNoBinding { .. } => 3,
        });
        bytes.extend_from_slice(&self.store);
        bytes.extend_from_slice(&self.reclaim_attempt);
        bytes.extend_from_slice(&self.manifest_record.allocation_epoch());
        bytes.extend_from_slice(&self.manifest_record.ordinal().to_le_bytes());
        bytes.extend_from_slice(&self.manifest_frame_sha256);
        bytes.extend_from_slice(&self.source_basis_digest);
        match self.proof {
            OriginalDropProofV1::ProvenNoEffect {
                idempotency,
                fingerprint,
                reserved,
            } => {
                bytes.extend_from_slice(&idempotency);
                bytes.extend_from_slice(&fingerprint);
                let reserved = reserved.expect("V2 positive proof admits an exact Reserved record");
                bytes.extend_from_slice(&reserved.record.allocation_epoch());
                bytes.extend_from_slice(&reserved.record.ordinal().to_le_bytes());
                bytes.extend_from_slice(&reserved.frame_sha256);
            }
            OriginalDropProofV1::NeverReserved => bytes.extend_from_slice(&[0; 120]),
            OriginalDropProofV1::RecoveredNoBinding {
                idempotency,
                fingerprint,
                reserved,
            } => {
                bytes.extend_from_slice(&idempotency);
                bytes.extend_from_slice(&fingerprint);
                bytes.extend_from_slice(&reserved.record.allocation_epoch());
                bytes.extend_from_slice(&reserved.record.ordinal().to_le_bytes());
                bytes.extend_from_slice(&reserved.frame_sha256);
            }
        }
        bytes.extend_from_slice(&self.source_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_sha256);
        bytes.extend_from_slice(&self.retained_metadata_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.publication.to_le_bytes());
        debug_assert_eq!(bytes.len(), WIRE_BYTES);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, crate::BlobManifestResidueCleanupDenial> {
        use crate::BlobManifestResidueCleanupDenial::{InvalidBinding, Malformed};
        if bytes.len() != WIRE_BYTES
            || bytes[..8] != (BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len() as u64).to_le_bytes()
            || &bytes[8..8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len()]
                != BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN
        {
            return Err(Malformed);
        }
        let mut cursor = 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len();
        let phase = match take::<1>(bytes, &mut cursor)[0] {
            1 => crate::BlobManifestResidueCleanupPhaseV1::Intent,
            2 => crate::BlobManifestResidueCleanupPhaseV1::Completed,
            _ => return Err(Malformed),
        };
        let proof_tag = take::<1>(bytes, &mut cursor)[0];
        let store = take::<16>(bytes, &mut cursor);
        let attempt = take::<16>(bytes, &mut cursor);
        let epoch = take::<16>(bytes, &mut cursor);
        let ordinal = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let manifest = PersistedRecordIdentity::new(epoch, ordinal).ok_or(InvalidBinding)?;
        let manifest_sha = take::<32>(bytes, &mut cursor);
        let basis = take::<32>(bytes, &mut cursor);
        let idempotency = take::<32>(bytes, &mut cursor);
        let fingerprint = take::<32>(bytes, &mut cursor);
        let reserved_epoch = take::<16>(bytes, &mut cursor);
        let reserved_ordinal = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let reserved_sha = take::<32>(bytes, &mut cursor);
        let proof = match proof_tag {
            1 => OriginalDropProofV1::ProvenNoEffect {
                idempotency,
                fingerprint,
                reserved: ReservedDropRecordV1::new(
                    PersistedRecordIdentity::new(reserved_epoch, reserved_ordinal)
                        .ok_or(InvalidBinding)?,
                    reserved_sha,
                ),
            },
            2 if idempotency == [0; 32]
                && fingerprint == [0; 32]
                && reserved_epoch == [0; 16]
                && reserved_ordinal == 0
                && reserved_sha == [0; 32] =>
            {
                OriginalDropProofV1::NeverReserved
            }
            3 => OriginalDropProofV1::RecoveredNoBinding {
                idempotency,
                fingerprint,
                reserved: ReservedDropRecordV1::new(
                    PersistedRecordIdentity::new(reserved_epoch, reserved_ordinal)
                        .ok_or(InvalidBinding)?,
                    reserved_sha,
                )
                .ok_or(InvalidBinding)?,
            },
            _ => return Err(InvalidBinding),
        };
        let source = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate_sha = take::<32>(bytes, &mut cursor);
        let metadata = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let publication = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let mut value = Self::intent(
            store,
            attempt,
            manifest,
            manifest_sha,
            basis,
            proof,
            source,
            candidate,
            candidate_sha,
            metadata,
            publication,
        )?;
        value.phase = phase;
        Ok(value)
    }
}

fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let end = *cursor + N;
    let value = bytes[*cursor..end]
        .try_into()
        .expect("fixed wire length admitted");
    *cursor = end;
    value
}
