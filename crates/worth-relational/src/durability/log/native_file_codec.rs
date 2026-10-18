use std::fs;
use std::path::Path;

use crate::durability::data::{DurabilityError, RecoveryFailureClass};

use super::local_store::{DurableCheckpointFile, DurableSegmentFile, DurableStoreManifestFile};
pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK4\n";

mod checkpoint_encoding;
pub(crate) use checkpoint_encoding::{encode_checkpoint, encode_payload};

pub(crate) fn read_store_manifest_file(
    path: &Path,
) -> Result<DurableStoreManifestFile, DurabilityError> {
    let bytes = fs::read(path).map_err(super::local_store::io_error)?;
    rmp_serde::from_slice(&bytes).map_err(|error| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "failed to decode durable native file {}: {error}",
                path.display()
            ),
        )
    })
}

pub(crate) fn write_store_manifest_file(
    path: &Path,
    file: &DurableStoreManifestFile,
) -> Result<(), DurabilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(super::local_store::io_error)?;
    }
    let temp_path = path.with_extension("tmp");
    let bytes = rmp_serde::to_vec_named(file).map_err(|error| {
        DurabilityError::new(
            RecoveryFailureClass::DurableIoFailure,
            format!(
                "failed to encode durable native file {}: {error}",
                path.display()
            ),
        )
    })?;
    fs::write(&temp_path, bytes).map_err(super::local_store::io_error)?;
    fs::rename(&temp_path, path).map_err(super::local_store::io_error)?;
    Ok(())
}

mod segment_frames;
#[cfg(test)]
pub(crate) use segment_frames::write as write_segment_file;
pub(crate) use segment_frames::{
    append as append_segment_bytes, inventory as read_segment_inventory,
    preflight as preflight_segments, read as read_segment_file,
};

pub(crate) fn write_checkpoint_file(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    path: &Path,
    checkpoint: &crate::durability::data::DurableCheckpoint,
) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
    execution
        .check_live()
        .map_err(|stop| crate::durability::data::RelationalRecoveryStop::Request(stop.into()))?;
    let (bytes, _) = encode_checkpoint(checkpoint, execution.allocation_policy())
        .map_err(crate::durability::data::RelationalRecoveryStop::from)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(super::local_store::io_error)?;
    }
    let temporary = path.with_extension("tmp");
    let mut output = fs::File::create(&temporary).map_err(super::local_store::io_error)?;
    use std::io::Write;
    output
        .write_all(bytes.bytes())
        .map_err(super::local_store::io_error)?;
    output.sync_all().map_err(super::local_store::io_error)?;
    drop(output);
    execution
        .check_live()
        .map_err(|stop| crate::durability::data::RelationalRecoveryStop::Request(stop.into()))?;
    fs::rename(temporary, path).map_err(super::local_store::io_error)?;
    Ok(())
}

mod text_readmission;
pub(crate) use text_readmission::{decode_checkpoint, read_checkpoint_file};
