use super::checksum::calculate;
use super::field_code::{artifact_parts, decode_artifact};
use super::target_shape::valid_target_shape;
use super::{
    PhysicalWorkArtifactCode, PhysicalWorkCheckpointActionCode, PhysicalWorkObligationIdentity,
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};

const MAGIC: &[u8; 8] = b"WPEFFECT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkObligationV6 {
    store_identity: [u8; 16],
    identity: PhysicalWorkObligationIdentity,
    operation_code: PhysicalWorkObligationOperationCode,
    target: PhysicalWorkObligationTargetCode,
    payload_digest: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkObligationV6Denial {
    LengthMismatch,
    WrongMagic,
    UnsupportedVersion(u8),
    ReservedFieldNonZero,
    ChecksumMismatch,
    InvalidIdentity,
    UnknownOperation(u8),
    InvalidTarget,
}

impl PhysicalWorkObligationV6 {
    pub fn new(
        store_identity: [u8; 16],
        runtime: u64,
        generation: u64,
        operation: u64,
        operation_code: PhysicalWorkObligationOperationCode,
        target: PhysicalWorkObligationTargetCode,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<Self, PhysicalWorkObligationV6Denial> {
        use core::num::NonZeroU64;

        if store_identity == [0; 16] {
            return Err(PhysicalWorkObligationV6Denial::InvalidIdentity);
        }
        let runtime =
            NonZeroU64::new(runtime).ok_or(PhysicalWorkObligationV6Denial::InvalidIdentity)?;
        let generation =
            NonZeroU64::new(generation).ok_or(PhysicalWorkObligationV6Denial::InvalidIdentity)?;
        let operation =
            NonZeroU64::new(operation).ok_or(PhysicalWorkObligationV6Denial::InvalidIdentity)?;
        Self::from_raw_store_and_identity(
            store_identity,
            PhysicalWorkObligationIdentity::new(runtime, generation, operation),
            operation_code,
            target,
            payload_digest,
        )
    }

    pub fn from_identity(
        store: crate::store_namespace::StableStoreIdentity,
        identity: PhysicalWorkObligationIdentity,
        operation_code: PhysicalWorkObligationOperationCode,
        target: PhysicalWorkObligationTargetCode,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<Self, PhysicalWorkObligationV6Denial> {
        Self::from_raw_store_and_identity(
            store.bytes(),
            identity,
            operation_code,
            target,
            payload_digest,
        )
    }

    fn from_raw_store_and_identity(
        store_identity: [u8; 16],
        identity: PhysicalWorkObligationIdentity,
        operation_code: PhysicalWorkObligationOperationCode,
        target: PhysicalWorkObligationTargetCode,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<Self, PhysicalWorkObligationV6Denial> {
        if !valid_target_shape(target, payload_digest.is_some()) {
            return Err(PhysicalWorkObligationV6Denial::InvalidTarget);
        }
        Ok(Self {
            store_identity,
            identity,
            operation_code,
            target,
            payload_digest,
        })
    }
    pub const fn identity(self) -> PhysicalWorkObligationIdentity {
        self.identity
    }
    pub const fn store_identity(self) -> [u8; 16] {
        self.store_identity
    }
    pub const fn runtime(self) -> u64 {
        self.identity.runtime().get()
    }
    pub const fn generation(self) -> u64 {
        self.identity.generation().get()
    }
    pub const fn operation(self) -> u64 {
        self.identity.operation().get()
    }
    pub const fn operation_code(self) -> PhysicalWorkObligationOperationCode {
        self.operation_code
    }
    pub const fn target(self) -> PhysicalWorkObligationTargetCode {
        self.target
    }
    pub const fn payload_digest(self) -> Option<[u8; 32]> {
        self.payload_digest
    }
}

pub fn encode_physical_work_obligation_v6(
    value: PhysicalWorkObligationV6,
) -> [u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES] {
    let mut record = [0; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES];
    record[..8].copy_from_slice(MAGIC);
    record[8] = PHYSICAL_WORK_OBLIGATION_V6_VERSION;
    record[9] = value.operation_code as u8;
    record[16..32].copy_from_slice(&value.store_identity);
    record[32..40].copy_from_slice(&value.runtime().to_le_bytes());
    record[40..48].copy_from_slice(&value.generation().to_le_bytes());
    record[48..56].copy_from_slice(&value.operation().to_le_bytes());
    encode_target(value.target, &mut record);
    if let Some(digest) = value.payload_digest {
        record[105] = 1;
        record[72..104].copy_from_slice(&digest);
    }
    let checksum = calculate(&record);
    record[128..].copy_from_slice(&checksum);
    record
}

pub fn decode_physical_work_obligation_v6(
    record: &[u8],
) -> Result<PhysicalWorkObligationV6, PhysicalWorkObligationV6Denial> {
    if record.len() != PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES {
        return Err(PhysicalWorkObligationV6Denial::LengthMismatch);
    }
    if &record[..8] != MAGIC {
        return Err(PhysicalWorkObligationV6Denial::WrongMagic);
    }
    if record[8] != PHYSICAL_WORK_OBLIGATION_V6_VERSION {
        return Err(PhysicalWorkObligationV6Denial::UnsupportedVersion(
            record[8],
        ));
    }
    if record[10..16].iter().any(|byte| *byte != 0)
        || record[107..112].iter().any(|byte| *byte != 0)
    {
        return Err(PhysicalWorkObligationV6Denial::ReservedFieldNonZero);
    }
    let fixed_record: &[u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES] = record
        .try_into()
        .expect("physical-work length was checked before checksum validation");
    if calculate(fixed_record) != record[128..] {
        return Err(PhysicalWorkObligationV6Denial::ChecksumMismatch);
    }
    let operation_code = PhysicalWorkObligationOperationCode::decode(record[9])
        .ok_or(PhysicalWorkObligationV6Denial::UnknownOperation(record[9]))?;
    let payload_digest = match record[105] {
        0 => None,
        1 => Some(record[72..104].try_into().expect("fixed digest")),
        _ => return Err(PhysicalWorkObligationV6Denial::InvalidTarget),
    };
    let value = PhysicalWorkObligationV6::new(
        record[16..32].try_into().expect("fixed store identity"),
        read_u64(record, 32),
        read_u64(record, 40),
        read_u64(record, 48),
        operation_code,
        decode_target(record, payload_digest.is_some())?,
        payload_digest,
    )?;
    Ok(value)
}

fn encode_target(target: PhysicalWorkObligationTargetCode, record: &mut [u8; 160]) {
    match target {
        PhysicalWorkObligationTargetCode::Range {
            artifact,
            offset,
            byte_count,
        } => {
            record[104] = 1;
            write_interval(record, offset, byte_count);
            write_artifact(record, artifact);
        }
        PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(artifact) => {
            record[104] = 2;
            write_artifact(record, artifact);
        }
        PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(artifact) => {
            record[104] = 3;
            write_artifact(record, artifact);
        }
        PhysicalWorkObligationTargetCode::CatalogReplacement(artifact) => {
            record[104] = 4;
            write_artifact(record, artifact);
        }
        PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization => record[104] = 5,
        PhysicalWorkObligationTargetCode::WalArtifactInterval {
            segment,
            generation,
            offset,
            byte_count,
        } => {
            record[104] = 6;
            write_interval(record, offset, byte_count);
            write_pair(record, segment, generation);
        }
        PhysicalWorkObligationTargetCode::Checkpoint { sequence, action } => {
            record[104] = 7;
            write_pair(record, sequence, 0);
            encode_checkpoint_action(record, action);
        }
        PhysicalWorkObligationTargetCode::WalSegmentReclamation {
            segment,
            generation,
        } => {
            record[104] = 8;
            write_pair(record, segment, generation);
        }
    }
}

fn decode_target(
    record: &[u8],
    has_digest: bool,
) -> Result<PhysicalWorkObligationTargetCode, PhysicalWorkObligationV6Denial> {
    let offset = read_u64(record, 56);
    let count = read_u64(record, 64);
    let first = read_u64(record, 112);
    let second = read_u64(record, 120);
    let empty = offset == 0 && count == 0;
    let value = match record[104] {
        1 if has_digest && count > 0 && offset.checked_add(count).is_some() => {
            PhysicalWorkObligationTargetCode::Range {
                artifact: decode_artifact(record[106], first, second)
                    .ok_or(PhysicalWorkObligationV6Denial::InvalidTarget)?,
                offset,
                byte_count: count,
            }
        }
        2 if empty && !has_digest => PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(
            decode_artifact(record[106], first, second)
                .ok_or(PhysicalWorkObligationV6Denial::InvalidTarget)?,
        ),
        3 if empty && !has_digest => {
            PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(
                decode_artifact(record[106], first, second)
                    .ok_or(PhysicalWorkObligationV6Denial::InvalidTarget)?,
            )
        }
        4 if empty && !has_digest => PhysicalWorkObligationTargetCode::CatalogReplacement(
            decode_artifact(record[106], first, second)
                .ok_or(PhysicalWorkObligationV6Denial::InvalidTarget)?,
        ),
        5 if empty && !has_digest && record[106] == 0 && first == 0 && second == 0 => {
            PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization
        }
        6 if record[106] == 0
            && first > 0
            && second > 0
            && count > 0
            && offset.checked_add(count).is_some()
            && has_digest =>
        {
            PhysicalWorkObligationTargetCode::WalArtifactInterval {
                segment: first,
                generation: second,
                offset,
                byte_count: count,
            }
        }
        7 if first > 0 && second == 0 => PhysicalWorkObligationTargetCode::Checkpoint {
            sequence: first,
            action: decode_checkpoint_action(record[106], offset, count, has_digest)
                .ok_or(PhysicalWorkObligationV6Denial::InvalidTarget)?,
        },
        8 if empty && !has_digest && record[106] == 0 && first > 0 && second > 0 => {
            PhysicalWorkObligationTargetCode::WalSegmentReclamation {
                segment: first,
                generation: second,
            }
        }
        _ => return Err(PhysicalWorkObligationV6Denial::InvalidTarget),
    };
    Ok(value)
}

fn encode_checkpoint_action(record: &mut [u8; 160], action: PhysicalWorkCheckpointActionCode) {
    match action {
        PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count } => {
            record[106] = 1;
            write_interval(record, 0, byte_count);
        }
        PhysicalWorkCheckpointActionCode::AppendCandidate { offset, byte_count } => {
            record[106] = 2;
            write_interval(record, offset, byte_count);
        }
        PhysicalWorkCheckpointActionCode::SynchronizeCandidate => record[106] = 3,
        PhysicalWorkCheckpointActionCode::RemoveCandidate => record[106] = 4,
        PhysicalWorkCheckpointActionCode::PublishCandidate => record[106] = 5,
        PhysicalWorkCheckpointActionCode::SynchronizeNamespace => record[106] = 6,
    }
}

fn decode_checkpoint_action(
    tag: u8,
    offset: u64,
    count: u64,
    digest: bool,
) -> Option<PhysicalWorkCheckpointActionCode> {
    match tag {
        1 if offset == 0 && count > 0 && digest => {
            Some(PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count: count })
        }
        2 if count > 0 && offset.checked_add(count).is_some() && digest => {
            Some(PhysicalWorkCheckpointActionCode::AppendCandidate {
                offset,
                byte_count: count,
            })
        }
        3 if offset == 0 && count == 0 && !digest => {
            Some(PhysicalWorkCheckpointActionCode::SynchronizeCandidate)
        }
        4 if offset == 0 && count == 0 && !digest => {
            Some(PhysicalWorkCheckpointActionCode::RemoveCandidate)
        }
        5 if offset == 0 && count == 0 && !digest => {
            Some(PhysicalWorkCheckpointActionCode::PublishCandidate)
        }
        6 if offset == 0 && count == 0 && !digest => {
            Some(PhysicalWorkCheckpointActionCode::SynchronizeNamespace)
        }
        _ => None,
    }
}

fn write_artifact(record: &mut [u8; 160], artifact: PhysicalWorkArtifactCode) {
    let (tag, first, second) = artifact_parts(artifact);
    record[106] = tag;
    write_pair(record, first, second);
}
fn write_interval(record: &mut [u8; 160], offset: u64, count: u64) {
    record[56..64].copy_from_slice(&offset.to_le_bytes());
    record[64..72].copy_from_slice(&count.to_le_bytes());
}
fn write_pair(record: &mut [u8; 160], first: u64, second: u64) {
    record[112..120].copy_from_slice(&first.to_le_bytes());
    record[120..128].copy_from_slice(&second.to_le_bytes());
}
fn read_u64(record: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(record[offset..offset + 8].try_into().expect("fixed field"))
}
