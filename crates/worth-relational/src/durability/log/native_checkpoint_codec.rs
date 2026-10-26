//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};

pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK10\n";

mod checkpoint_decoding;
mod checkpoint_encoding;
mod checkpoint_wire;
mod cold_checkpoint;
mod embedded_commit;
mod expanded_checkpoint;
mod manifest_capture;
mod manifest_validation;
pub(crate) mod member_frame;
mod member_source;
pub(crate) use checkpoint_decoding::{decode_checkpoint, decode_checkpoint_from_source};
pub(crate) use checkpoint_encoding::encode_captured;
#[cfg(test)]
pub(crate) use checkpoint_encoding::encode_checkpoint;
pub(crate) use manifest_capture::CapturedManifest;

/// A checkpoint decoded from native bytes. Decoding readmits managed text and
/// the canonical envelopes; recovery still checks continuity before use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedNativeCheckpoint {
    pub(crate) checkpoint: DurableCheckpoint,
}

#[cfg(test)]
mod kind_authentication_tests;
#[cfg(test)]
mod tests;
