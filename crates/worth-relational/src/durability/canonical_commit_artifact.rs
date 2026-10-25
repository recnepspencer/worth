//! Relational's sealed, self-describing byte form of one positioned
//! canonical commit. Another storage medium persists these bytes opaquely;
//! Relational alone owns the frame, its version, the payload digest and the
//! readmission door back to canonical authority.
//!
//! Frame (all integers little-endian, header is fixed width):
//!
//! | bytes  | field                                   |
//! |--------|-----------------------------------------|
//! | 0..8   | magic `WRTHRCCA`                        |
//! | 8..10  | artifact format version (`u16`)         |
//! | 10..18 | `PatchStreamPosition` (`u64`)           |
//! | 18..26 | `CommitId` (`u64`)                      |
//! | 26..58 | SHA-256 of the payload                  |
//! | 58..66 | payload length (`u64`)                  |
//! | 66..   | payload: MessagePack named map          |
//!
//! The payload is `{position, canonical}` where `canonical` is the
//! authoritative envelope projection. The position is inside the digested
//! payload so a rewritten header position cannot pass readmission.

mod encode_denial;
mod encoding;
mod frame;
mod frame_refusal;
mod payload_digest;
mod payload_wire;
mod readmission;
mod readmission_refusal;
mod retained_export;

pub use encode_denial::RelationalCanonicalCommitArtifactEncodeDenial;
pub use frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal;
pub use payload_digest::RelationalCanonicalCommitArtifactDigest;
pub use readmission_refusal::RelationalCanonicalCommitArtifactRefusal;
pub use retained_export::RelationalCanonicalCommitArtifacts;

use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use frame::CanonicalCommitArtifactHeader;

/// One positioned canonical commit as frame-verified bytes.
///
/// Holding an artifact proves only that its frame, version, length and
/// payload digest are intact. It confers no canonical authority: callers
/// obtain a commit only through [`RelationalCanonicalCommitArtifact::readmit`].
/// Clones share the immutable byte backing.
pub struct RelationalCanonicalCommitArtifact {
    bytes: ExecutionImmutableBytes,
    header: CanonicalCommitArtifactHeader,
}

impl RelationalCanonicalCommitArtifact {
    /// The artifact format version this build writes and readmits. Bytes
    /// carrying any other version are refused, never reinterpreted.
    pub const FORMAT_VERSION: u16 = frame::FORMAT_VERSION;

    /// Accept bytes from an untrusted medium after checking the frame only:
    /// magic, version, header width, payload length and payload digest. The
    /// payload is not decoded and no authority is granted.
    ///
    /// Cost: one SHA-256 pass over the payload.
    pub fn from_untrusted_bytes(
        bytes: impl Into<Box<[u8]>>,
    ) -> Result<Self, RelationalCanonicalCommitArtifactFrameRefusal> {
        let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into()));
        let (header, _) = CanonicalCommitArtifactHeader::verify(bytes.bytes())?;
        Ok(Self { bytes, header })
    }

    pub fn position(&self) -> PatchStreamPosition {
        self.header.position
    }

    pub fn commit_id(&self) -> CommitId {
        self.header.commit_id
    }

    pub fn payload_digest(&self) -> RelationalCanonicalCommitArtifactDigest {
        self.header.digest
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    pub fn into_bytes(self) -> ExecutionImmutableBytes {
        self.bytes
    }
}

impl Clone for RelationalCanonicalCommitArtifact {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            header: self.header,
        }
    }
}

impl std::fmt::Debug for RelationalCanonicalCommitArtifact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalCanonicalCommitArtifact")
            .field("position", &self.header.position)
            .field("commit_id", &self.header.commit_id)
            .field("payload_digest", &self.header.digest)
            .field("byte_len", &self.as_bytes().len())
            .finish()
    }
}

impl PartialEq for RelationalCanonicalCommitArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for RelationalCanonicalCommitArtifact {}
