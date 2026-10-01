use crate::{PersistedRecordIdentity, ReleasedDropPredecessorV1};

use super::ReleaseCustodyHeadDenial;
use crate::manifest::durable_root_routing::{decode_identity, encode_identity};

pub(super) const KEY_BYTES: usize = 24;
pub(super) const ENTRY_BYTES: usize = 312;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReleaseCustodyHeadKeyV1 {
    object: [u8; 16],
    generation: u64,
}

impl ReleaseCustodyHeadKeyV1 {
    pub fn new(object: [u8; 16], generation: u64) -> Option<Self> {
        if object == [0; 16] || generation == 0 {
            None
        } else {
            Some(Self { object, generation })
        }
    }
    pub const fn object(self) -> [u8; 16] {
        self.object
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadEntryV1 {
    key: ReleaseCustodyHeadKeyV1,
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    source_basis_digest: [u8; 32],
    predecessor: Option<ReleasedDropPredecessorV1>,
    source_root_generation: u64,
    cumulative_dropped: u64,
    terminal: bool,
}

impl ReleaseCustodyHeadEntryV1 {
    pub const ENCODED_BYTES: usize = ENTRY_BYTES;
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: ReleaseCustodyHeadKeyV1,
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        source_basis_digest: [u8; 32],
        predecessor: Option<ReleasedDropPredecessorV1>,
        source_root_generation: u64,
        cumulative_dropped: u64,
        terminal: bool,
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        if [
            descriptor_frame_sha256,
            manifest_frame_sha256,
            reservation_frame_sha256,
            source_basis_digest,
        ]
        .contains(&[0; 32])
            || descriptor_record == manifest_record
            || descriptor_record == reservation_record
            || manifest_record == reservation_record
            || source_root_generation == 0
            || cumulative_dropped == 0
        {
            return Err(ReleaseCustodyHeadDenial::Malformed);
        }
        Ok(Self {
            key,
            descriptor_record,
            descriptor_frame_sha256,
            manifest_record,
            manifest_frame_sha256,
            reservation_record,
            reservation_frame_sha256,
            source_basis_digest,
            predecessor,
            source_root_generation,
            cumulative_dropped,
            terminal,
        })
    }
    pub const fn key(self) -> ReleaseCustodyHeadKeyV1 {
        self.key
    }
    pub const fn descriptor_record(self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn reservation_record(self) -> PersistedRecordIdentity {
        self.reservation_record
    }
    pub const fn reservation_frame_sha256(self) -> [u8; 32] {
        self.reservation_frame_sha256
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis_digest
    }
    pub const fn predecessor(self) -> Option<ReleasedDropPredecessorV1> {
        self.predecessor
    }
    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub const fn cumulative_dropped(self) -> u64 {
        self.cumulative_dropped
    }
    pub const fn terminal(self) -> bool {
        self.terminal
    }

    pub fn encode_into(self, target: &mut [u8; ENTRY_BYTES]) {
        *target = [0; ENTRY_BYTES];
        encode_key(self.key, (&mut target[..KEY_BYTES]).try_into().unwrap());
        encode_identity(&mut target[24..48], self.descriptor_record);
        target[48..80].copy_from_slice(&self.descriptor_frame_sha256);
        encode_identity(&mut target[80..104], self.manifest_record);
        target[104..136].copy_from_slice(&self.manifest_frame_sha256);
        encode_identity(&mut target[136..160], self.reservation_record);
        target[160..192].copy_from_slice(&self.reservation_frame_sha256);
        target[192..224].copy_from_slice(&self.source_basis_digest);
        if let Some(prior) = self.predecessor {
            target[224] = 1;
            encode_identity(&mut target[232..256], prior.descriptor_record());
            target[256..288].copy_from_slice(&prior.descriptor_frame_sha256());
        }
        target[288..296].copy_from_slice(&self.source_root_generation.to_le_bytes());
        target[296..304].copy_from_slice(&self.cumulative_dropped.to_le_bytes());
        target[304] = u8::from(self.terminal);
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ReleaseCustodyHeadDenial> {
        if bytes.len() != ENTRY_BYTES || bytes[225..232] != [0; 7] || bytes[305..312] != [0; 7] {
            return Err(ReleaseCustodyHeadDenial::Malformed);
        }
        let predecessor = match bytes[224] {
            0 if bytes[232..288] == [0; 56] => None,
            1 => Some(
                ReleasedDropPredecessorV1::new(
                    decode_identity(&bytes[232..256]).ok_or(ReleaseCustodyHeadDenial::Malformed)?,
                    bytes[256..288].try_into().unwrap(),
                )
                .map_err(|_| ReleaseCustodyHeadDenial::Malformed)?,
            ),
            _ => return Err(ReleaseCustodyHeadDenial::Malformed),
        };
        let terminal = match bytes[304] {
            0 => false,
            1 => true,
            _ => return Err(ReleaseCustodyHeadDenial::Malformed),
        };
        Self::new(
            decode_key(&bytes[..24])?,
            decode_identity(&bytes[24..48]).ok_or(ReleaseCustodyHeadDenial::Malformed)?,
            bytes[48..80].try_into().unwrap(),
            decode_identity(&bytes[80..104]).ok_or(ReleaseCustodyHeadDenial::Malformed)?,
            bytes[104..136].try_into().unwrap(),
            decode_identity(&bytes[136..160]).ok_or(ReleaseCustodyHeadDenial::Malformed)?,
            bytes[160..192].try_into().unwrap(),
            bytes[192..224].try_into().unwrap(),
            predecessor,
            u64::from_le_bytes(bytes[288..296].try_into().unwrap()),
            u64::from_le_bytes(bytes[296..304].try_into().unwrap()),
            terminal,
        )
    }
}

pub(super) fn encode_key(key: ReleaseCustodyHeadKeyV1, target: &mut [u8; KEY_BYTES]) {
    target[..16].copy_from_slice(&key.object);
    target[16..24].copy_from_slice(&key.generation.to_le_bytes());
}

pub(super) fn decode_key(
    bytes: &[u8],
) -> Result<ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadDenial> {
    if bytes.len() != KEY_BYTES {
        return Err(ReleaseCustodyHeadDenial::Malformed);
    }
    ReleaseCustodyHeadKeyV1::new(
        bytes[..16].try_into().unwrap(),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
    )
    .ok_or(ReleaseCustodyHeadDenial::Malformed)
}
