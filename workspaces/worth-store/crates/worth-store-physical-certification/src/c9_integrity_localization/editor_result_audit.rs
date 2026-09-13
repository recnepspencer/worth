use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::corruption_operator::RootCorruptionOperation;
use super::frame_checksum::{checksum_is_valid, encoded_payload_length, FRAME_HEADER_BYTES};
use super::{CleanRootArtifactRecord, DeclaredRootCorruption, RootArtifactIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditorResultAudit {
    target: RootArtifactIdentity,
    declaration_identity: [u8; 32],
    before_sha256: [u8; 32],
    after_sha256: Option<[u8; 32]>,
    changed_ranges: Vec<Range<u64>>,
    created_path: Option<PathBuf>,
    removed_path: Option<PathBuf>,
    checksum_valid_after_edit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EditorAuditDenial {
    UnknownTarget,
    DeclarationMismatch,
    BaselineChanged,
    SourceChanged,
    TargetRead,
    TargetWrite,
    NamespaceMutation,
    ContractViolation,
}

pub(super) struct AppliedEdit {
    after: Option<Vec<u8>>,
    source: Option<Vec<u8>>,
    created_path: Option<PathBuf>,
    removed_path: Option<PathBuf>,
}

pub(super) type TreeSnapshot = BTreeMap<PathBuf, [u8; 32]>;

pub(super) struct TreeSnapshotObservation {
    pub(super) snapshot: TreeSnapshot,
    pub(super) files_read: u64,
    pub(super) bytes_read: u64,
}

impl AppliedEdit {
    pub(super) fn new(
        after: Option<Vec<u8>>,
        source: Option<Vec<u8>>,
        created_path: Option<PathBuf>,
        removed_path: Option<PathBuf>,
    ) -> Self {
        Self {
            after,
            source,
            created_path,
            removed_path,
        }
    }
}

pub(super) fn capture_tree(root: &Path) -> Result<TreeSnapshotObservation, EditorAuditDenial> {
    let mut snapshot = BTreeMap::new();
    let mut files_read = 0;
    let mut bytes_read = 0;
    capture_directory(root, root, &mut snapshot, &mut files_read, &mut bytes_read)?;
    Ok(TreeSnapshotObservation {
        snapshot,
        files_read,
        bytes_read,
    })
}

pub(super) fn audit_editor_result(
    record: &CleanRootArtifactRecord,
    edit: &DeclaredRootCorruption,
    before: Vec<u8>,
    applied: AppliedEdit,
    before_tree: TreeSnapshot,
    after_tree: TreeSnapshot,
) -> Result<EditorResultAudit, EditorAuditDenial> {
    validate_operation_result(record, edit.operation(), &before, &applied)?;
    validate_namespace_delta(record, edit.operation(), &before_tree, &after_tree)?;
    let changed_ranges = applied
        .after
        .as_deref()
        .map(|after| changed_byte_ranges(&before, after))
        .unwrap_or_default();
    let after_sha256 = applied
        .after
        .as_deref()
        .map(|after| Sha256::digest(after).into());
    let checksum_valid_after_edit = applied.after.as_deref().map(checksum_is_valid);
    Ok(EditorResultAudit {
        target: edit.target(),
        declaration_identity: edit.identity(),
        before_sha256: Sha256::digest(before).into(),
        after_sha256,
        changed_ranges,
        created_path: applied.created_path,
        removed_path: applied.removed_path,
        checksum_valid_after_edit,
    })
}

fn validate_operation_result(
    record: &CleanRootArtifactRecord,
    operation: &RootCorruptionOperation,
    before: &[u8],
    applied: &AppliedEdit,
) -> Result<(), EditorAuditDenial> {
    let after = applied.after.as_deref();
    let valid = match operation {
        RootCorruptionOperation::CoveredByteFlip { offset, .. }
        | RootCorruptionOperation::ChecksumFieldFlip { offset, .. } => after.is_some_and(|after| {
            changed_byte_ranges(before, after)
                == std::iter::once(*offset..*offset + 1).collect::<Vec<_>>()
                && !checksum_is_valid(after)
        }),
        RootCorruptionOperation::FramingLengthLie {
            encoded_payload_length: lied_payload_length,
        } => after.is_some_and(|after| {
            after.len() == before.len()
                && encoded_payload_length(after) == Some(*lied_payload_length)
                && usize::try_from(*lied_payload_length).ok()
                    != Some(after.len() - FRAME_HEADER_BYTES)
                && checksum_is_valid(after)
                && changes_stay_within(
                    &changed_byte_ranges(before, after),
                    &[record.length_range(), record.checksum_range()],
                )
        }),
        RootCorruptionOperation::ScopeSubstitution { .. } => after.is_some_and(|after| {
            applied.source.as_deref() == Some(after)
                && after.len() == before.len()
                && after != before
                && checksum_is_valid(after)
        }),
        RootCorruptionOperation::PointerCorruption { range, replacement } => {
            after.is_some_and(|after| {
                after[range.start as usize..range.end as usize] == replacement.to_le_bytes()
                    && checksum_is_valid(after)
                    && changes_stay_within(
                        &changed_byte_ranges(before, after),
                        &[range.clone(), record.checksum_range()],
                    )
            })
        }
        RootCorruptionOperation::StrictPrefixTruncation { retained_length } => {
            after.is_some_and(|after| {
                *retained_length > 0
                    && *retained_length < before.len() as u64
                    && after == &before[..*retained_length as usize]
            })
        }
        RootCorruptionOperation::ArtifactRemoval => {
            after.is_none() && applied.removed_path.as_deref() == Some(record.relative_path())
        }
        RootCorruptionOperation::ArtifactDuplication { destination } => {
            after == Some(before)
                && applied.created_path.as_ref() == Some(destination)
                && checksum_is_valid(before)
        }
        RootCorruptionOperation::UnsupportedFormatVersion { range, value } => {
            after.is_some_and(|after| {
                after[range.start as usize..range.end as usize] == value.to_le_bytes()
                    && checksum_is_valid(after)
                    && changes_stay_within(
                        &changed_byte_ranges(before, after),
                        &[range.clone(), record.checksum_range()],
                    )
            })
        }
    };
    valid
        .then_some(())
        .ok_or(EditorAuditDenial::ContractViolation)
}

fn validate_namespace_delta(
    record: &CleanRootArtifactRecord,
    operation: &RootCorruptionOperation,
    before: &TreeSnapshot,
    after: &TreeSnapshot,
) -> Result<(), EditorAuditDenial> {
    let mut expected = before.clone();
    match operation {
        RootCorruptionOperation::ArtifactRemoval => {
            expected.remove(record.relative_path());
        }
        RootCorruptionOperation::ArtifactDuplication { destination } => {
            let digest = *before
                .get(record.relative_path())
                .ok_or(EditorAuditDenial::BaselineChanged)?;
            expected.insert(destination.clone(), digest);
        }
        _ => {
            let observed = *after
                .get(record.relative_path())
                .ok_or(EditorAuditDenial::NamespaceMutation)?;
            expected.insert(record.relative_path().to_path_buf(), observed);
        }
    }
    (expected == *after)
        .then_some(())
        .ok_or(EditorAuditDenial::NamespaceMutation)
}

fn changed_byte_ranges(before: &[u8], after: &[u8]) -> Vec<Range<u64>> {
    let compared = before.len().min(after.len());
    let mut changed = Vec::new();
    let mut start = None;
    for index in 0..compared {
        if before[index] != after[index] {
            start.get_or_insert(index);
        } else if let Some(range_start) = start.take() {
            changed.push(range_start as u64..index as u64);
        }
    }
    if let Some(range_start) = start {
        changed.push(range_start as u64..compared as u64);
    }
    if before.len() != after.len() {
        changed.push(compared as u64..before.len().max(after.len()) as u64);
    }
    changed
}

fn changes_stay_within(changes: &[Range<u64>], allowed: &[Range<u64>]) -> bool {
    !changes.is_empty()
        && changes.iter().all(|change| {
            allowed
                .iter()
                .any(|range| range.start <= change.start && range.end >= change.end)
        })
}

fn capture_directory(
    root: &Path,
    directory: &Path,
    snapshot: &mut TreeSnapshot,
    files_read: &mut u64,
    bytes_read: &mut u64,
) -> Result<(), EditorAuditDenial> {
    for entry in std::fs::read_dir(directory).map_err(|_| EditorAuditDenial::TargetRead)? {
        let path = entry.map_err(|_| EditorAuditDenial::TargetRead)?.path();
        let metadata =
            std::fs::symlink_metadata(&path).map_err(|_| EditorAuditDenial::TargetRead)?;
        if metadata.file_type().is_symlink() {
            return Err(EditorAuditDenial::NamespaceMutation);
        }
        if metadata.is_dir() {
            capture_directory(root, &path, snapshot, files_read, bytes_read)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| EditorAuditDenial::NamespaceMutation)?
                .to_path_buf();
            let bytes = std::fs::read(path).map_err(|_| EditorAuditDenial::TargetRead)?;
            *files_read += 1;
            *bytes_read += bytes.len() as u64;
            snapshot.insert(relative, Sha256::digest(bytes).into());
        } else {
            return Err(EditorAuditDenial::NamespaceMutation);
        }
    }
    Ok(())
}

impl EditorResultAudit {
    pub(crate) const fn target(&self) -> RootArtifactIdentity {
        self.target
    }
    pub(crate) const fn declaration_identity(&self) -> [u8; 32] {
        self.declaration_identity
    }
    pub(crate) const fn before_sha256(&self) -> [u8; 32] {
        self.before_sha256
    }
    pub(crate) const fn after_sha256(&self) -> Option<[u8; 32]> {
        self.after_sha256
    }
    pub(crate) fn changed_ranges(&self) -> &[Range<u64>] {
        &self.changed_ranges
    }
    pub(crate) fn created_path(&self) -> Option<&Path> {
        self.created_path.as_deref()
    }
    pub(crate) fn removed_path(&self) -> Option<&Path> {
        self.removed_path.as_deref()
    }
    pub(crate) const fn checksum_valid_after_edit(&self) -> Option<bool> {
        self.checksum_valid_after_edit
    }
}
