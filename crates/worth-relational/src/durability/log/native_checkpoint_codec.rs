//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};

// This grammar joins authenticated Original-member inventory and binary digests
// with current custody, kind counts and exact index bases. Neither parent
// grammar (CHK18 or CHK20.1), nor master's distinct CHK21, is an input format.
pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK22\n";
fn checkpoint_payload(bytes: &[u8]) -> Result<&[u8], DurabilityError> {
    bytes.strip_prefix(CHECKPOINT_MAGIC).ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            format!(
                "unsupported native checkpoint frame; {} required",
                String::from_utf8_lossy(CHECKPOINT_MAGIC).trim_end()
            ),
        )
    })
}

mod checkpoint_decoding;
mod checkpoint_encoding;
mod complete_encoding;
pub(crate) use complete_encoding::{canonical_inline_size, encode_complete};
mod checkpoint_wire;
mod cold_checkpoint;
mod dictionary_capture;
mod expanded_checkpoint;
mod lineage_event_capture;
mod lineage_event_directory;
mod manifest_capture;
mod manifest_validation;
pub(crate) mod member_frame;
mod member_sequence;
mod member_source;
pub(crate) use checkpoint_decoding::{decode_checkpoint, decode_checkpoint_from_source};
pub(crate) use checkpoint_decoding::{decode_checkpoint_with_image, decode_interim_with_image};
#[cfg(test)]
pub(crate) use checkpoint_encoding::encode_checkpoint;
pub(crate) use checkpoint_encoding::{encode_captured, CheckpointEncodingControl};
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

mod inspection;
pub(crate) use inspection::inspect as inspect_checkpoint;
