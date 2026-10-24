//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};

pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK6\n";

mod checkpoint_decoding;
mod checkpoint_encoding;
pub(crate) use checkpoint_decoding::decode_checkpoint;
pub(crate) use checkpoint_encoding::encode_checkpoint;

/// A checkpoint decoded from native bytes. Decoding readmits managed text and
/// the canonical envelopes; recovery still checks continuity before use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedNativeCheckpoint {
    pub(crate) checkpoint: DurableCheckpoint,
}
