use serde::Deserialize;

use crate::durability::data::{DurabilityError, RecoveryFailureClass};

/// Native checkpoint image format written by this build. Format 1 is the first
/// image that carries retired branch names; absent means an older image.
pub(super) const NATIVE_CHECKPOINT_FORMAT_VERSION: u16 = 1;

/// Refuse every image this build did not write before any section readmits.
/// The native store is disposable, so older formats are refused, not migrated.
pub(super) fn readmit_native_checkpoint_format(format: u16) -> Result<(), DurabilityError> {
    if format == NATIVE_CHECKPOINT_FORMAT_VERSION {
        return Ok(());
    }
    Err(unsupported_format(format))
}

/// Classify an image whose full decode failed.
///
/// An image of another format usually cannot decode as the current image at
/// all, so the failure alone does not say which it was. The format field is
/// read on its own here: an image that states another format, or none, is an
/// unsupported format, and everything else is a corrupt image of this format.
/// Recovery falls back past corruption and never past an unsupported format.
pub(in crate::durability::log) fn undecodable_checkpoint(
    bytes: &[u8],
    error: impl std::fmt::Display,
) -> DurabilityError {
    match rmp_serde::from_slice::<NativeFormatProbeFile>(bytes) {
        Ok(probe) if probe.checkpoint.native_format != NATIVE_CHECKPOINT_FORMAT_VERSION => {
            unsupported_format(probe.checkpoint.native_format)
        }
        _ => DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("failed to decode native checkpoint: {error}"),
        ),
    }
}

/// The envelope every format shares: a map whose `checkpoint` entry is a map
/// that states `native_format`. That much is frozen across formats, because it
/// is the only thing a build can read of an image it did not write.
#[derive(Deserialize)]
struct NativeFormatProbeFile {
    checkpoint: NativeFormatProbe,
}

#[derive(Deserialize)]
struct NativeFormatProbe {
    #[serde(default)]
    native_format: u16,
}

fn unsupported_format(format: u16) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::UnsupportedCheckpointFormat,
        format!(
            "unsupported native checkpoint format {format}; this build reads only format {NATIVE_CHECKPOINT_FORMAT_VERSION}"
        ),
    )
}
