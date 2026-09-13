use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_physical_backend::BackendTargetProfile;

use super::{ReplicationProgressStoreError, Snapshot, StoredReplicationPeerProgress};
use crate::{
    DurabilityReplayIdentity, DurabilityReplayKind, ReplicationLineageIdentity, ReplicationPeerId,
    ReplicationSourceEpoch,
};

const MAGIC: &[u8; 8] = b"WORTHRPS";
const VERSION: u16 = 2;
const HEADER_BYTES: usize = 68;
const RECORD_FIXED_BYTES: usize = 108;
const CHECKSUM_BYTES: usize = 32;

pub(super) enum SnapshotSlot {
    Missing,
    Torn,
    Corrupt,
    Valid(Snapshot),
}

pub(super) fn encode(
    generation: u64,
    authority: StoreCurrentAuthorityIdentity,
    records: &BTreeMap<ReplicationPeerId, StoredReplicationPeerProgress>,
) -> Result<Vec<u8>, ReplicationProgressStoreError> {
    let mut payload = Vec::new();
    for progress in records.values() {
        encode_record(progress, &mut payload)?;
    }
    let mut snapshot = Vec::with_capacity(HEADER_BYTES + payload.len() + CHECKSUM_BYTES);
    snapshot.extend_from_slice(MAGIC);
    snapshot.extend_from_slice(&VERSION.to_le_bytes());
    snapshot.extend_from_slice(&(HEADER_BYTES as u16).to_le_bytes());
    snapshot.extend_from_slice(&generation.to_le_bytes());
    snapshot.extend_from_slice(&(records.len() as u64).to_le_bytes());
    snapshot.extend_from_slice(&authority.fingerprint());
    snapshot.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    snapshot.extend_from_slice(&payload);
    let checksum = Sha256::digest(&snapshot);
    snapshot.extend_from_slice(&checksum);
    Ok(snapshot)
}

fn encode_record(
    progress: &StoredReplicationPeerProgress,
    output: &mut Vec<u8>,
) -> Result<(), ReplicationProgressStoreError> {
    let peer = progress.peer_id.as_str().as_bytes();
    let lineage = progress.lineage.as_str().as_bytes();
    let digest = progress.replay.digest().as_bytes();
    let total = RECORD_FIXED_BYTES
        .checked_add(peer.len())
        .and_then(|bytes| bytes.checked_add(lineage.len()))
        .and_then(|bytes| bytes.checked_add(digest.len()))
        .ok_or(ReplicationProgressStoreError::Io)?;
    output.extend_from_slice(
        &u32::try_from(total)
            .map_err(|_| ReplicationProgressStoreError::Io)?
            .to_le_bytes(),
    );
    output.extend_from_slice(&progress.source_epoch.get().to_le_bytes());
    output.push(replay_kind_code(progress.replay.kind()));
    output.push(profile_code(progress.replay.profile()));
    output.extend_from_slice(&[0; 2]);
    output.extend_from_slice(&progress.replay.first_lsn().to_le_bytes());
    output.extend_from_slice(&progress.replay.last_lsn().to_le_bytes());
    output.extend_from_slice(&progress.current_authority.fingerprint());
    output.extend_from_slice(&progress.security_scope_fingerprint);
    append_length(peer.len(), output)?;
    append_length(lineage.len(), output)?;
    append_length(digest.len(), output)?;
    output.extend_from_slice(peer);
    output.extend_from_slice(lineage);
    output.extend_from_slice(digest);
    Ok(())
}

pub(super) fn read(path: &Path) -> Result<SnapshotSlot, ReplicationProgressStoreError> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SnapshotSlot::Missing)
        }
        Err(_) => return Err(ReplicationProgressStoreError::Io),
    };
    if bytes.len() < HEADER_BYTES + CHECKSUM_BYTES {
        return Ok(SnapshotSlot::Torn);
    }
    if &bytes[..8] != MAGIC
        || read_u16(&bytes, 8)? != VERSION
        || read_u16(&bytes, 10)? as usize != HEADER_BYTES
    {
        return Ok(SnapshotSlot::Corrupt);
    }
    let payload_bytes =
        usize::try_from(read_u64(&bytes, 60)?).map_err(|_| ReplicationProgressStoreError::Io)?;
    let expected_bytes = HEADER_BYTES
        .checked_add(payload_bytes)
        .and_then(|length| length.checked_add(CHECKSUM_BYTES))
        .ok_or(ReplicationProgressStoreError::Io)?;
    if bytes.len() < expected_bytes {
        return Ok(SnapshotSlot::Torn);
    }
    if bytes.len() > expected_bytes {
        return Ok(SnapshotSlot::Corrupt);
    }
    let checksum_at = bytes.len() - CHECKSUM_BYTES;
    if Sha256::digest(&bytes[..checksum_at])[..] != bytes[checksum_at..] {
        return Ok(SnapshotSlot::Corrupt);
    }
    let generation = read_u64(&bytes, 12)?;
    let record_count =
        usize::try_from(read_u64(&bytes, 20)?).map_err(|_| ReplicationProgressStoreError::Io)?;
    let authority = bytes[28..60].try_into().expect("fixed authority width");
    let Ok(records) = decode_records(&bytes[HEADER_BYTES..checksum_at], record_count) else {
        return Ok(SnapshotSlot::Corrupt);
    };
    Ok(SnapshotSlot::Valid(Snapshot {
        generation,
        authority: StoreCurrentAuthorityIdentity::from_persisted_fingerprint(authority),
        records,
    }))
}

fn append_length(length: usize, output: &mut Vec<u8>) -> Result<(), ReplicationProgressStoreError> {
    output.extend_from_slice(
        &u32::try_from(length)
            .map_err(|_| ReplicationProgressStoreError::Io)?
            .to_le_bytes(),
    );
    Ok(())
}

fn decode_records(
    bytes: &[u8],
    expected_count: usize,
) -> Result<BTreeMap<ReplicationPeerId, StoredReplicationPeerProgress>, ReplicationProgressStoreError>
{
    let mut records = BTreeMap::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let total = read_u32(bytes, offset)? as usize;
        let record_end = offset
            .checked_add(total)
            .ok_or(ReplicationProgressStoreError::Io)?;
        let record = bytes
            .get(offset..record_end)
            .ok_or(ReplicationProgressStoreError::Io)?;
        if total < RECORD_FIXED_BYTES {
            return Err(ReplicationProgressStoreError::Io);
        }
        let epoch = read_u64(record, 4)?;
        let first_lsn = read_u64(record, 16)?;
        let last_lsn = read_u64(record, 24)?;
        let current_authority = record[32..64].try_into().expect("fixed authority width");
        let security_scope_fingerprint = record[64..96].try_into().expect("fixed scope width");
        let peer_len = read_u32(record, 96)? as usize;
        let lineage_len = read_u32(record, 100)? as usize;
        let digest_len = read_u32(record, 104)? as usize;
        let described_total = RECORD_FIXED_BYTES
            .checked_add(peer_len)
            .and_then(|length| length.checked_add(lineage_len))
            .and_then(|length| length.checked_add(digest_len))
            .ok_or(ReplicationProgressStoreError::Io)?;
        if described_total != total {
            return Err(ReplicationProgressStoreError::Io);
        }
        let peer_start = RECORD_FIXED_BYTES;
        let lineage_start = peer_start + peer_len;
        let digest_start = lineage_start + lineage_len;
        let peer_id = ReplicationPeerId::admit(text(&record[peer_start..lineage_start])?)
            .ok_or(ReplicationProgressStoreError::Io)?;
        records.insert(
            peer_id.clone(),
            StoredReplicationPeerProgress {
                peer_id,
                source_epoch: ReplicationSourceEpoch::admit(epoch)
                    .ok_or(ReplicationProgressStoreError::Io)?,
                lineage: ReplicationLineageIdentity::admit(text(
                    &record[lineage_start..digest_start],
                )?)
                .ok_or(ReplicationProgressStoreError::Io)?,
                current_authority: StoreCurrentAuthorityIdentity::from_persisted_fingerprint(
                    current_authority,
                ),
                security_scope_fingerprint,
                replay: DurabilityReplayIdentity::new(
                    decode_publication(record[12])?,
                    decode_profile(record[13])?,
                    text(&record[digest_start..])?,
                    first_lsn,
                    last_lsn,
                )
                .map_err(|_| ReplicationProgressStoreError::Io)?,
            },
        );
        offset = record_end;
    }
    (records.len() == expected_count)
        .then_some(records)
        .ok_or(ReplicationProgressStoreError::Io)
}

fn text(bytes: &[u8]) -> Result<String, ReplicationProgressStoreError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| ReplicationProgressStoreError::Io)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ReplicationProgressStoreError> {
    Ok(u16::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ReplicationProgressStoreError> {
    Ok(u32::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ReplicationProgressStoreError> {
    Ok(u64::from_le_bytes(read_array(bytes, offset)?))
}

fn read_array<const N: usize>(
    bytes: &[u8],
    offset: usize,
) -> Result<[u8; N], ReplicationProgressStoreError> {
    bytes
        .get(offset..offset + N)
        .ok_or(ReplicationProgressStoreError::Io)?
        .try_into()
        .map_err(|_| ReplicationProgressStoreError::Io)
}

const fn replay_kind_code(kind: DurabilityReplayKind) -> u8 {
    match kind {
        DurabilityReplayKind::WalFrame => 1,
        DurabilityReplayKind::Checkpoint => 2,
        DurabilityReplayKind::Manifest => 3,
    }
}

fn decode_publication(code: u8) -> Result<DurabilityReplayKind, ReplicationProgressStoreError> {
    match code {
        1 => Ok(DurabilityReplayKind::WalFrame),
        2 => Ok(DurabilityReplayKind::Checkpoint),
        3 => Ok(DurabilityReplayKind::Manifest),
        _ => Err(ReplicationProgressStoreError::Io),
    }
}

const fn profile_code(profile: BackendTargetProfile) -> u8 {
    match profile {
        BackendTargetProfile::SimulatedStrictDurable => 1,
        BackendTargetProfile::PosixFileFsyncDirSync => 2,
        BackendTargetProfile::WindowsFlushFileBuffers => 3,
        BackendTargetProfile::MmapFlushNotDurabilityCertified => 4,
        BackendTargetProfile::AdversarialLostFlush => 5,
        BackendTargetProfile::AdversarialReorderedFlush => 6,
    }
}

fn decode_profile(code: u8) -> Result<BackendTargetProfile, ReplicationProgressStoreError> {
    match code {
        1 => Ok(BackendTargetProfile::SimulatedStrictDurable),
        2 => Ok(BackendTargetProfile::PosixFileFsyncDirSync),
        3 => Ok(BackendTargetProfile::WindowsFlushFileBuffers),
        4 => Ok(BackendTargetProfile::MmapFlushNotDurabilityCertified),
        5 => Ok(BackendTargetProfile::AdversarialLostFlush),
        6 => Ok(BackendTargetProfile::AdversarialReorderedFlush),
        _ => Err(ReplicationProgressStoreError::Io),
    }
}
