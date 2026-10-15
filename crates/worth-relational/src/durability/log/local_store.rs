mod store_io;
pub(crate) use store_io::{lock_store, lock_store_for_recovery, preflight_store};

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::durability::data::{
    DurabilityError, DurableCheckpoint, DurableCheckpointId, DurableIntegrityStatus,
    DurableSegmentId, DurableSegmentManifest, DurableStore, DurableStoreLayout,
    RecoveryFailureClass,
};
use crate::durability::log::native_file_codec::{
    read_segment_file, read_segment_inventory, read_store_manifest_file, write_store_manifest_file,
};
use crate::durability::log::persisted_canonical_commit::PersistedCanonicalCommit;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableStoreManifestFile {
    pub(crate) store: DurableStore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableSegmentFile {
    pub(crate) entries: Vec<PersistedCanonicalCommit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DurableCheckpointFile {
    pub(crate) checkpoint: DurableCheckpoint,
}

pub(crate) fn ensure_loaded_store(
    runtime: &impl crate::capabilities::RuntimeConfigSource,
) -> Result<DurableStore, DurabilityError> {
    let Some(layout) = runtime
        .runtime_config()
        .durability
        .policy
        .store_layout
        .clone()
    else {
        return Err(DurabilityError::new(
            RecoveryFailureClass::DurableIoFailure,
            "persisted durability mode requires a durable store layout",
        ));
    };
    load_or_initialize_store(layout)
}

pub(crate) fn load_store_from_disk(
    runtime: &impl crate::capabilities::RuntimeConfigSource,
) -> Result<DurableStore, DurabilityError> {
    let Some(layout) = runtime
        .runtime_config()
        .durability
        .policy
        .store_layout
        .clone()
    else {
        return Err(DurabilityError::new(
            RecoveryFailureClass::DurableIoFailure,
            "persisted durability mode requires a durable store layout",
        ));
    };
    load_or_initialize_store(layout)
}

pub(crate) fn persist_store_manifest(store: &DurableStore) -> Result<(), DurabilityError> {
    ensure_store_dirs(&store.layout)?;
    write_store_manifest_file(
        &manifest_path(&store.layout),
        &DurableStoreManifestFile {
            store: store.clone(),
        },
    )
}

pub(crate) fn current_segment_ids(store: Option<&DurableStore>) -> Vec<DurableSegmentId> {
    store
        .map(|store| {
            store
                .segments
                .iter()
                .map(|segment| segment.segment_id)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn manifest_path(layout: &DurableStoreLayout) -> PathBuf {
    layout.root_path.join("manifest.worth-store")
}

pub(crate) fn segment_file_path(
    layout: &DurableStoreLayout,
    segment_id: DurableSegmentId,
) -> PathBuf {
    layout
        .root_path
        .join("segments")
        .join(format!("segment-{}.worth-segment", segment_id.0))
}

pub(crate) fn checkpoint_file_path(
    layout: &DurableStoreLayout,
    checkpoint_id: DurableCheckpointId,
) -> PathBuf {
    layout
        .root_path
        .join("checkpoints")
        .join(format!("checkpoint-{}.worth-checkpoint", checkpoint_id.0))
}

pub(crate) fn ensure_store_dirs(layout: &DurableStoreLayout) -> Result<(), DurabilityError> {
    fs::create_dir_all(layout.root_path.join("segments")).map_err(io_error)?;
    fs::create_dir_all(layout.root_path.join("checkpoints")).map_err(io_error)?;
    Ok(())
}

pub(crate) fn load_or_initialize_store(
    layout: DurableStoreLayout,
) -> Result<DurableStore, DurabilityError> {
    ensure_store_dirs(&layout)?;
    let manifest = manifest_path(&layout);
    if manifest.exists() {
        return refresh_store_segments_from_disk(read_store_manifest_file(&manifest)?.store);
    }
    let store = DurableStore {
        layout: layout.clone(),
        segments: Vec::new(),
        checkpoints: Vec::new(),
    };
    write_store_manifest_file(
        &manifest,
        &DurableStoreManifestFile {
            store: store.clone(),
        },
    )?;
    refresh_store_segments_from_disk(store)
}

fn refresh_store_segments_from_disk(
    mut store: DurableStore,
) -> Result<DurableStore, DurabilityError> {
    let segments_dir = store.layout.root_path.join("segments");
    let mut refreshed_segments = fs::read_dir(&segments_dir)
        .map_err(io_error)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "worth-segment"))
        .filter_map(|path| {
            let stem = path.file_stem()?.to_str()?;
            let suffix = stem.strip_prefix("segment-")?;
            let segment_id = suffix.parse::<u64>().ok()?;
            Some((DurableSegmentId(segment_id), path))
        })
        .map(|(segment_id, path)| {
            let existing = store
                .segments
                .iter()
                .find(|segment| segment.segment_id == segment_id)
                .cloned();
            let (first_commit_id, last_commit_id, commit_count, integrity) =
                match read_segment_inventory(&path) {
                    Ok(entries) => (
                        entries.first().copied(),
                        entries.last().copied(),
                        entries.len(),
                        DurableIntegrityStatus::Verified,
                    ),
                    Err(_) => (
                        existing
                            .as_ref()
                            .and_then(|segment| segment.first_commit_id),
                        existing.as_ref().and_then(|segment| segment.last_commit_id),
                        existing
                            .as_ref()
                            .map(|segment| segment.commit_count)
                            .unwrap_or(0),
                        DurableIntegrityStatus::Corrupt,
                    ),
                };
            DurableSegmentManifest {
                segment_id,
                path,
                first_commit_id,
                last_commit_id,
                commit_count,
                runtime_name: existing
                    .as_ref()
                    .map(|segment| segment.runtime_name.clone())
                    .unwrap_or_default(),
                profile: existing
                    .as_ref()
                    .map(|segment| segment.profile)
                    .unwrap_or(crate::config::data::RelationalRuntimeProfile::CertificationCore),
                schema_version: existing
                    .as_ref()
                    .map(|segment| segment.schema_version)
                    .unwrap_or(crate::schema::data::SchemaVersionId(0)),
                integrity,
            }
        })
        .collect::<Vec<_>>();
    refreshed_segments.sort_by_key(|segment| segment.segment_id);
    store.segments = refreshed_segments;
    Ok(store)
}

pub(crate) fn read_segment_entries(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    path: &Path,
) -> Result<
    Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    crate::durability::data::RelationalRecoveryStop,
> {
    read_segment_file(execution, path)?
        .entries
        .into_iter()
        .map(|entry| {
            entry.readmit().map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptSegment, detail).into()
            })
        })
        .collect()
}

pub(crate) fn io_error(error: std::io::Error) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::DurableIoFailure, error.to_string())
}

#[cfg(test)]
mod legacy_segment_tests {
    use super::*;

    #[test]
    fn native_reader_refuses_old_segment_before_interpreting_text() {
        let bytes = decode_hex(include_str!(
            "../../../tests/fixtures/worth_query_9_16_1_1_native_segment.hex"
        ));
        let path = std::env::temp_dir().join(format!(
            "worth-old-segment-{}.worth-segment",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).unwrap();
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let error = read_segment_entries(execution, &path).unwrap_err();
        assert_eq!(
            crate::tests::support::expect_durability_failure(error).class,
            RecoveryFailureClass::UnsupportedLegacySemantics
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
    }

    fn decode_hex(encoded: &str) -> Vec<u8> {
        let digits = encoded
            .bytes()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect::<Vec<_>>();
        assert_eq!(digits.len() % 2, 0, "fixture hex has complete bytes");
        digits
            .chunks_exact(2)
            .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
            .collect()
    }

    fn hex_digit(digit: u8) -> u8 {
        match digit {
            b'0'..=b'9' => digit - b'0',
            b'a'..=b'f' => digit - b'a' + 10,
            _ => panic!("fixture contains a non-hex digit"),
        }
    }
}
