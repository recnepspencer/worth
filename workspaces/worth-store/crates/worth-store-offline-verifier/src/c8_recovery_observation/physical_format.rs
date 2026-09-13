use sha2::{Digest, Sha256};

use super::observer_evidence_accumulation::{
    RecoveryObserverArtifactEvidence, RecoveryObserverResidueObservation,
};
use super::{checkpoint_observation, durable_observation, wal_observation};

pub(super) const DURABLE_FRAME_MAGIC: &[u8; 8] = b"WRC5FRM\0";
pub(super) const DURABLE_FRAME_HEADER_BYTES: usize = 48;
pub(super) const WAL_MAGIC: &[u8; 8] = b"WORTHWAL";
pub(super) const CHECKPOINT_MAGIC: &[u8; 8] = b"WORTHCKP";
pub(super) const CHECKPOINT_STREAM_MAGIC: &[u8; 8] = b"WCP7REC\0";

pub(super) fn observe(path: &str, bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    // A checkpoint candidate is not namespace authority until its selector
    // publication settles. Even a structurally complete candidate is residue
    // to an independent observer while it remains under staging/.
    if path.starts_with("staging/") && path.ends_with(".candidate") {
        return residue(bytes);
    }
    match bytes.get(..8) {
        Some(magic) if magic == WAL_MAGIC && selected_wal_path(path) => {
            wal_observation::observe(bytes)
        }
        Some(magic) if magic == CHECKPOINT_MAGIC => checkpoint_observation::observe(bytes),
        Some(magic) if magic == CHECKPOINT_STREAM_MAGIC => {
            checkpoint_observation::observe_stream(bytes)
        }
        Some(magic) if magic == DURABLE_FRAME_MAGIC => durable_observation::observe(bytes),
        _ => residue(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::observe;

    #[test]
    fn unreferenced_checkpoint_candidate_is_observed_as_residue() {
        let evidence = observe("staging/checkpoint.candidate", b"candidate-bytes");
        assert_eq!(evidence.residue.bytes, 15);
        assert_ne!(evidence.residue.digest, [0; 32]);
        assert!(evidence.checkpoint.is_none());
    }
}

fn selected_wal_path(path: &str) -> bool {
    path.strip_prefix("families/wal/")
        .is_some_and(|relative| relative.ends_with(".wal"))
}

pub(super) fn residue(bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    RecoveryObserverArtifactEvidence {
        residue: RecoveryObserverResidueObservation {
            bytes: bytes.len() as u64,
            digest: Sha256::digest(bytes).into(),
        },
        ..RecoveryObserverArtifactEvidence::empty()
    }
}

pub(super) fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)
        .and_then(|value| value.try_into().ok())
        .map(u64::from_le_bytes)
}

pub(super) fn digest_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(super) struct DurableFrame<'a> {
    pub(super) kind: u8,
    pub(super) format: [u8; 10],
    pub(super) identity: u64,
    pub(super) page_lsn: u64,
    pub(super) payload: &'a [u8],
}

pub(super) fn durable_frame(bytes: &[u8]) -> Option<DurableFrame<'_>> {
    if bytes.len() < DURABLE_FRAME_HEADER_BYTES
        || bytes.get(..8)? != &DURABLE_FRAME_MAGIC[..]
        || !(1..=11).contains(bytes.get(8)?)
        || *bytes.get(9)? != 2
        || read_u16(bytes, 20)? as usize != DURABLE_FRAME_HEADER_BYTES
        || bytes.get(22..24)? != [0; 2]
    {
        return None;
    }
    let format = bytes.get(10..20)?;
    let page_bytes = read_u32(format, 2)?;
    if read_u16(format, 0)? != 1
        || !matches!(page_bytes, 16_384 | 32_768 | 65_536)
        || format.get(6..10)? != [1, 1, 1, 24]
    {
        return None;
    }
    let payload_bytes = usize::try_from(read_u32(bytes, 24)?).ok()?;
    let total = DURABLE_FRAME_HEADER_BYTES.checked_add(payload_bytes)?;
    if bytes.len() != total {
        return None;
    }
    let checksum = read_u32(bytes, 44)?;
    let mut covered = Vec::with_capacity(44 + payload_bytes);
    covered.extend_from_slice(bytes.get(..44)?);
    covered.extend_from_slice(bytes.get(DURABLE_FRAME_HEADER_BYTES..)?);
    if worth_store_physical_format::durable_artifact_checksum(&covered) != checksum {
        return None;
    }
    Some(DurableFrame {
        kind: *bytes.get(8)?,
        format: format.try_into().ok()?,
        identity: read_u64(bytes, 28)?,
        page_lsn: read_u64(bytes, 36)?,
        payload: bytes.get(DURABLE_FRAME_HEADER_BYTES..)?,
    })
}
