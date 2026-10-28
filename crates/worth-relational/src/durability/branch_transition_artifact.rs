//! Relational's sealed, self-describing byte form of one positioned branch
//! lifecycle transition (Milestone 2 D8). Another storage medium persists
//! these bytes opaquely; Relational alone owns the frame, its version, the
//! payload digest and the readmission door back to a positioned transition.
//!
//! Frame (all integers little-endian, header is fixed width):
//!
//! | bytes  | field                                        |
//! |--------|----------------------------------------------|
//! | 0..8   | magic `WRTHRBTA`                             |
//! | 8..10  | artifact format version (`u16`)              |
//! | 10..18 | `RelationalBranchTransitionSequence` (`u64`) |
//! | 18..26 | `follows`: `PatchStreamPosition` (`u64`)     |
//! | 26..58 | SHA-256 of the payload                       |
//! | 58..66 | payload length (`u64`)                       |
//! | 66..   | payload: MessagePack named map               |
//!
//! The payload is `{sequence, follows, transition}`. Both header fields are
//! inside the digested payload, so a rewritten header cannot pass
//! readmission.

mod encode_denial;
mod encoding;
mod frame;
mod frame_refusal;
mod payload_digest;
mod payload_wire;
mod readmission;
mod readmission_refusal;
mod retained_export;

pub use encode_denial::RelationalBranchTransitionArtifactEncodeDenial;
pub use frame_refusal::RelationalBranchTransitionArtifactFrameRefusal;
pub use payload_digest::RelationalBranchTransitionArtifactDigest;
pub use readmission_refusal::RelationalBranchTransitionArtifactRefusal;
pub use retained_export::RelationalBranchTransitionArtifacts;

use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

use crate::branch::RelationalBranchTransitionSequence;
use crate::publication::patch::data::PatchStreamPosition;
use frame::BranchTransitionArtifactHeader;

/// One positioned branch transition as frame-verified bytes.
///
/// Holding an artifact proves only that its frame, version, length and
/// payload digest are intact. It confers no authority: callers obtain a
/// transition only through [`RelationalBranchTransitionArtifact::readmit`].
/// Clones share the immutable byte backing.
pub struct RelationalBranchTransitionArtifact {
    bytes: ExecutionImmutableBytes,
    header: BranchTransitionArtifactHeader,
}

impl RelationalBranchTransitionArtifact {
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
    ) -> Result<Self, RelationalBranchTransitionArtifactFrameRefusal> {
        let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into()));
        let (header, _) = BranchTransitionArtifactHeader::verify(bytes.bytes())?;
        Ok(Self { bytes, header })
    }

    pub fn sequence(&self) -> RelationalBranchTransitionSequence {
        self.header.sequence
    }

    /// The performed commit position the transition follows.
    pub fn follows(&self) -> PatchStreamPosition {
        self.header.follows
    }

    pub fn payload_digest(&self) -> RelationalBranchTransitionArtifactDigest {
        self.header.digest
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    pub fn into_bytes(self) -> ExecutionImmutableBytes {
        self.bytes
    }
}

impl Clone for RelationalBranchTransitionArtifact {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            header: self.header,
        }
    }
}

impl std::fmt::Debug for RelationalBranchTransitionArtifact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalBranchTransitionArtifact")
            .field("sequence", &self.header.sequence)
            .field("follows", &self.header.follows)
            .field("payload_digest", &self.header.digest)
            .field("byte_len", &self.as_bytes().len())
            .finish()
    }
}

impl PartialEq for RelationalBranchTransitionArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for RelationalBranchTransitionArtifact {}
