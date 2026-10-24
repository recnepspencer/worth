//! Members of an anchored value: content-addressed records published with the
//! value in its root decision and carried, not rewritten, by later values.

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::PhysicalRecordId;

/// The fixed width of one locator in an anchored record's member table.
pub const ANCHORED_RECORD_MEMBER_LOCATOR_BYTES: usize = 16 + 8 + 32 + 8;

/// The most members one anchored value may list.
pub const MAXIMUM_ANCHORED_RECORD_MEMBERS: usize = 1 << 20;

/// Where one member of an anchored value lives and what it must hash to.
///
/// A locator names a member only while the value that lists it is current:
/// reading or carrying a locator is refused once its value is replaced
/// without it. Its key is its SHA-256.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnchoredRecordMemberLocator {
    record: PhysicalRecordId,
    sha256: [u8; 32],
    length: u64,
}

impl AnchoredRecordMemberLocator {
    pub(super) const fn new(record: PhysicalRecordId, sha256: [u8; 32], length: u64) -> Self {
        Self {
            record,
            sha256,
            length,
        }
    }

    pub const fn record(&self) -> PhysicalRecordId {
        self.record
    }

    /// The member's content key: SHA-256 of its bytes.
    pub const fn sha256(&self) -> [u8; 32] {
        self.sha256
    }

    pub const fn length(&self) -> u64 {
        self.length
    }

    /// The fixed-width form an owner may keep in its own payload.
    pub fn to_bytes(&self) -> [u8; ANCHORED_RECORD_MEMBER_LOCATOR_BYTES] {
        let mut bytes = [0; ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
        bytes[..16].copy_from_slice(&self.record.allocation_epoch());
        bytes[16..24].copy_from_slice(&self.record.ordinal().to_le_bytes());
        bytes[24..56].copy_from_slice(&self.sha256);
        bytes[56..].copy_from_slice(&self.length.to_le_bytes());
        bytes
    }

    /// Decodes the fixed-width form; `None` when it names no record. A decoded
    /// locator still reads only while a current value lists it.
    pub fn from_bytes(bytes: &[u8; ANCHORED_RECORD_MEMBER_LOCATOR_BYTES]) -> Option<Self> {
        let field = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("fixed"));
        let epoch = bytes[..16].try_into().expect("fixed field");
        let record = PersistedRecordIdentity::new(epoch, field(16))?;
        Some(Self {
            record: PhysicalRecordId::from_persisted(record),
            sha256: bytes[24..56].try_into().expect("fixed field"),
            length: field(56),
        })
    }
}

/// One member of a requested anchored value, in the caller's order.
pub enum AnchoredRecordMember {
    /// Content to write. `key` must be the SHA-256 of `bytes`.
    New { key: [u8; 32], bytes: Vec<u8> },
    /// A member of the anchor's current value, kept without rewriting.
    Carried {
        locator: AnchoredRecordMemberLocator,
    },
}

impl AnchoredRecordMember {
    /// The content identity a retry is compared by: key and length.
    pub(super) fn content(&self) -> ([u8; 32], u64) {
        match self {
            Self::New { key, bytes } => (*key, bytes.len() as u64),
            Self::Carried { locator } => (locator.sha256, locator.length),
        }
    }
}

impl std::fmt::Debug for AnchoredRecordMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New { bytes, .. } => f
                .debug_struct("New")
                .field("bytes", &bytes.len())
                .finish_non_exhaustive(),
            Self::Carried { locator } => {
                f.debug_struct("Carried").field("locator", locator).finish()
            }
        }
    }
}
