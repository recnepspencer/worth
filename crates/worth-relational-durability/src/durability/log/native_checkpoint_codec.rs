//! Relational's native checkpoint bytes: one self-describing frame a durable
//! medium stores opaquely and hands back for readmission.

use crate::durability::data::DurableCheckpoint;
use worth_relational_contracts::internal_port::durability::data::recovery_errors::{
    DurabilityError, RecoveryFailureClass,
};

// CHK24 is the single clean cutover from CHK23: the eager record-identity
// tables are stored as content-addressed paged members. No earlier grammar is
// an input format.
pub const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK24\n";
pub const CHECKPOINT_VERSION: u16 = 24;
const CHECKPOINT_FAMILY: &[u8] = b"WORTH-REL-CHK";

fn checkpoint_payload(bytes: &[u8]) -> Result<&[u8], DurabilityError> {
    if let Some(payload) = bytes.strip_prefix(CHECKPOINT_MAGIC) {
        return Ok(payload);
    }
    Err(match framed_checkpoint_version(bytes) {
        Some(found) => DurabilityError::new(
            RecoveryFailureClass::UnsupportedCheckpointVersion {
                found,
                supported: CHECKPOINT_VERSION,
            },
            format!(
                "native checkpoint frame CHK{found} is not supported; CHK{CHECKPOINT_VERSION} required"
            ),
        ),
        None => DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            format!(
                "unsupported native checkpoint frame; {} required",
                String::from_utf8_lossy(CHECKPOINT_MAGIC).trim_end()
            ),
        ),
    })
}

/// The unsupported major version of a `WORTH-REL-CHK<major>[.<minor>]\n` frame.
fn framed_checkpoint_version(bytes: &[u8]) -> Option<u16> {
    let label = bytes.strip_prefix(CHECKPOINT_FAMILY)?;
    let end = label.iter().take(12).position(|byte| *byte == b'\n')?;
    let version = &label[..end];
    let major = match version.iter().position(|byte| *byte == b'.') {
        Some(dot) => {
            let minor = &version[dot + 1..];
            if minor.is_empty() || !minor.iter().all(u8::is_ascii_digit) {
                return None;
            }
            &version[..dot]
        }
        None => version,
    };
    if major.is_empty() || major[0] == b'0' || !major.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let found: u16 = std::str::from_utf8(major).ok()?.parse().ok()?;
    (found != CHECKPOINT_VERSION).then_some(found)
}

pub mod checkpoint_decoding;
pub mod checkpoint_encoding;
pub mod complete_encoding;
pub(crate) use complete_encoding::encode_complete;
pub mod checkpoint_wire;
pub mod cold_checkpoint;
pub mod dictionary_capture;
pub mod expanded_checkpoint;
pub(crate) mod identity_pages;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_payload_refuses_other_versions_and_malformed_headers_distinctly() {
        assert!(checkpoint_payload(b"WORTH-REL-CHK24\npayload").is_ok());
        for (frame, expected) in [
            (b"WORTH-REL-CHK23\n".as_slice(), 23),
            (b"WORTH-REL-CHK23.1\n".as_slice(), 23),
            (b"WORTH-REL-CHK25\n".as_slice(), 25),
        ] {
            let err = checkpoint_payload(frame).unwrap_err();
            assert_eq!(
                err.class,
                RecoveryFailureClass::UnsupportedCheckpointVersion {
                    found: expected,
                    supported: CHECKPOINT_VERSION,
                }
            );
        }
        for malformed in [
            b"WORTH-REL-CHK24.1\n".as_slice(),
            b"WORTH-REL-CHK024\n".as_slice(),
            b"WORTH-REL-CHK0\n".as_slice(),
            b"WORTH-REL-CHK23.foo\n".as_slice(),
            b"WORTH-REL-CHK23.\n".as_slice(),
            b"WORTH-REL-CHK23.1.2\n".as_slice(),
            b"WORTH-REL-CHK+23\n".as_slice(),
        ] {
            let err = checkpoint_payload(malformed).unwrap_err();
            assert_eq!(
                err.class,
                RecoveryFailureClass::UnsupportedLegacySemantics,
                "expected UnsupportedLegacySemantics for {:?}",
                String::from_utf8_lossy(malformed)
            );
        }
    }
}
