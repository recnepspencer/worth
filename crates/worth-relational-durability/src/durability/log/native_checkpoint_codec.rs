//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::DurableCheckpoint;
use worth_relational_contracts::internal_port::durability::data::recovery_errors::{
    DurabilityError, RecoveryFailureClass,
};

// This grammar joins authenticated Original-member inventory and binary digests
// with current custody, kind counts and exact index bases. Neither parent
// grammar (CHK18 or CHK20.1), nor master's distinct CHK21, is an input format.
pub const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK23\n";
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

pub mod checkpoint_decoding;
pub mod checkpoint_encoding;
pub mod complete_encoding;
pub(crate) use complete_encoding::encode_complete;
pub mod checkpoint_wire;
pub mod cold_checkpoint;
pub mod dictionary_capture;
pub mod expanded_checkpoint;
pub mod lineage_event_capture;
pub mod lineage_event_directory;
pub mod manifest_capture;
pub mod manifest_validation;
pub mod member_frame;
pub mod member_sequence;
pub use checkpoint_decoding::decode_checkpoint_from_source;
#[doc(hidden)]
pub use checkpoint_decoding::decode_expanded_checkpoint;
pub(crate) use checkpoint_decoding::{decode_checkpoint_with_image, decode_manifest_directory};
#[doc(hidden)]
pub use checkpoint_encoding::encode_checkpoint_members;
pub(crate) use checkpoint_encoding::{
    encode_captured, populate_expanded_canonical_members, CheckpointEncodingControl,
};
pub(crate) use manifest_capture::CapturedManifest;

/// A checkpoint decoded from native bytes. Decoding readmits managed text and
/// the canonical envelopes; recovery still checks continuity before use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedNativeCheckpoint {
    pub checkpoint: DurableCheckpoint,
}

pub mod inspection;
pub(crate) use inspection::inspect as inspect_checkpoint;
