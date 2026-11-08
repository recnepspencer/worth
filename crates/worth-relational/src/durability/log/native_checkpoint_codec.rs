//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};

// Minor capability revision: current custody rows and v2 route-presence
// digests require a reader that understands successor custody. CHK21 is the
// separate paged-directory format, not this unchanged CHK20 body grammar.
pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK20.1\n";
const LEGACY_CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK20\n";

fn checkpoint_payload(bytes: &[u8]) -> Result<&[u8], DurabilityError> {
    bytes
        .strip_prefix(CHECKPOINT_MAGIC)
        .or_else(|| bytes.strip_prefix(LEGACY_CHECKPOINT_MAGIC))
        .ok_or_else(|| {
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
mod checkpoint_wire;
mod cold_checkpoint;
mod dictionary_capture;
mod embedded_commit;
mod expanded_checkpoint;
mod lineage_event_capture;
mod lineage_event_directory;
mod manifest_capture;
mod manifest_validation;
pub(crate) mod member_frame;
mod member_source;
pub(crate) use checkpoint_decoding::{decode_checkpoint, decode_checkpoint_from_source};
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
