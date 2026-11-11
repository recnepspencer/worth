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
//! | 26..58 | v8 composite payload commitment         |
//! | 58..66 | payload length (`u64`)                  |
//! | 66..   | payload: MessagePack named map          |
//!
//! Version 8 carries `{position, canonical, original_inputs}`. The canonical
//! projection and exact schema/policy input DATA have separate SHA-256 leaves;
//! the composite commitment also binds the header position and commit. See
//! `payload_digest` for its exact domain-separated grammar.

pub(crate) mod borrowed_payload;
#[cfg(test)]
mod checkpoint_body;
mod decoded_quote;
pub(crate) use decoded_quote::schema_image::{
    prepare as prepare_schema_image_decode, SchemaImageDecodeAdmission,
};
pub(crate) use decoded_quote::{decode_checkpoint_effect, CheckpointEffectData};
mod commitment_writer;
pub(crate) const MAXIMUM_ARTIFACT_BYTES: usize = 256 * 1024 * 1024;

mod encode_denial;
mod encoding;
mod frame;
mod frame_refusal;
pub(crate) mod input_commitment;
mod memory_publication;
pub(crate) use memory_publication::MemoryFramePreparationDenial;
mod prepared_payload;
pub(crate) use prepared_payload::PreparedOriginalCanonicalPayload;
mod payload_digest;
mod payload_wire;
mod readmission;
mod readmission_refusal;
mod request_agreement;
mod request_encoding;
pub(crate) use request_encoding::ControlledFrameCopyDenial;
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
    /// Bounded version eligibility only, never frame or payload admission.
    pub fn preflight_frame_version(
        prefix: &[u8],
    ) -> Result<(), RelationalCanonicalCommitArtifactFrameRefusal> {
        frame::validate_frame_version(prefix)
    }
    pub const PROTOCOL: worth_foundational::facade::BoundaryProtocolIdentity =
        worth_foundational::facade::BoundaryProtocolIdentity::new(
            "worth.relational.canonical-commit",
        );
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
        Self::from_owned_bytes(ExecutionImmutableBytes::from_external_bytes(Arc::new(
            bytes.into(),
        )))
    }

    /// Keep the original provider allocation and its intrinsic charge while
    /// checking this same frame. No encoding or byte-copy path participates.
    pub(crate) fn from_owned_bytes(
        bytes: ExecutionImmutableBytes,
    ) -> Result<Self, RelationalCanonicalCommitArtifactFrameRefusal> {
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
