use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::durability::data::{DurabilityError, RecoveryFailureClass};

use super::local_store::{DurableCheckpointFile, DurableSegmentFile, DurableStoreManifestFile};
use super::persisted_checkpoint::{
    PersistedDurableCheckpointFile, PersistedDurableCheckpointFileRef,
};

pub(crate) fn read_store_manifest_file(
    path: &Path,
) -> Result<DurableStoreManifestFile, DurabilityError> {
    read_native_file(path)
}

pub(crate) fn write_store_manifest_file(
    path: &Path,
    file: &DurableStoreManifestFile,
) -> Result<(), DurabilityError> {
    write_native_file(path, file)
}

pub(crate) fn read_segment_file(path: &Path) -> Result<DurableSegmentFile, DurabilityError> {
    read_native_file(path)
}

pub(crate) fn write_segment_file(
    path: &Path,
    file: &DurableSegmentFile,
) -> Result<(), DurabilityError> {
    write_native_file(path, file)
}

pub(crate) fn read_checkpoint_file(path: &Path) -> Result<DurableCheckpointFile, DurabilityError> {
    read_native_file::<PersistedDurableCheckpointFile>(path)?.readmit()
}

pub(crate) fn write_checkpoint_file(
    path: &Path,
    checkpoint: &crate::durability::data::DurableCheckpoint,
) -> Result<(), DurabilityError> {
    write_native_file(path, &PersistedDurableCheckpointFileRef::new(checkpoint))
}

pub(crate) fn encode_checkpoint(
    checkpoint: crate::durability::data::DurableCheckpoint,
) -> Result<Vec<u8>, DurabilityError> {
    rmp_serde::to_vec_named(&PersistedDurableCheckpointFileRef::new(&checkpoint)).map_err(|error| {
        DurabilityError::new(
            RecoveryFailureClass::DurableIoFailure,
            format!("failed to encode native checkpoint: {error}"),
        )
    })
}

pub(crate) fn decode_checkpoint(bytes: &[u8]) -> Result<DurableCheckpointFile, DurabilityError> {
    rmp_serde::from_slice::<PersistedDurableCheckpointFile>(bytes)
        .map_err(|error| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("failed to decode native checkpoint: {error}"),
            )
        })?
        .readmit()
}

fn read_native_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, DurabilityError> {
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

fn write_native_file<T: Serialize>(path: &Path, value: &T) -> Result<(), DurabilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(super::local_store::io_error)?;
    }
    let temp_path = path.with_extension("tmp");
    let bytes = rmp_serde::to_vec_named(value).map_err(|error| {
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
