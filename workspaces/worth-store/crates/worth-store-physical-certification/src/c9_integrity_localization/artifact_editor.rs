use std::path::Path;

use sha2::{Digest, Sha256};

use super::corruption_operator::RootCorruptionOperation;
use super::editor_result_audit::{audit_editor_result, capture_tree, AppliedEdit};
use super::frame_checksum::refresh_checksum;
use super::{
    CleanRootArtifactManifest, DeclaredRootCorruption, EditorAuditDenial, EditorResultAudit,
    RootLocalizationCounters,
};

pub(crate) fn apply_declared_corruption(
    row_root: &Path,
    manifest: &CleanRootArtifactManifest,
    edit: &DeclaredRootCorruption,
    counters: &mut RootLocalizationCounters,
) -> Result<EditorResultAudit, EditorAuditDenial> {
    if !edit.is_exact_for(manifest) {
        return Err(EditorAuditDenial::DeclarationMismatch);
    }
    let record = manifest
        .record(edit.target())
        .ok_or(EditorAuditDenial::UnknownTarget)?;
    let target_path = row_root.join(record.relative_path());
    let before = std::fs::read(&target_path).map_err(|_| EditorAuditDenial::TargetRead)?;
    if Sha256::digest(&before).as_slice() != record.content_sha256() {
        return Err(EditorAuditDenial::BaselineChanged);
    }
    let before_tree = capture_tree(row_root)?;
    let applied = apply_operation(row_root, record, edit.operation(), &target_path, &before)?;
    let after_tree = capture_tree(row_root)?;
    let audit = audit_editor_result(
        record,
        edit,
        before,
        applied,
        before_tree.snapshot,
        after_tree.snapshot,
    )?;
    counters.record_edit(
        1 + u64::from(matches!(
            edit.operation(),
            RootCorruptionOperation::ScopeSubstitution { .. }
        )) + before_tree.files_read
            + after_tree.files_read,
        edit_bytes_read(edit.operation(), record.exact_length())
            + before_tree.bytes_read
            + after_tree.bytes_read,
        edit_bytes_written(edit.operation(), record.exact_length()),
        edit_refreshes_checksum(edit.operation()),
        u64::from(audit.removed_path().is_some()),
        u64::from(audit.created_path().is_some()),
    );
    Ok(audit)
}

fn apply_operation(
    row_root: &Path,
    record: &super::CleanRootArtifactRecord,
    operation: &RootCorruptionOperation,
    target_path: &Path,
    before: &[u8],
) -> Result<AppliedEdit, EditorAuditDenial> {
    let mut after = before.to_vec();
    let mut source = None;
    let mut created_path = None;
    let mut removed_path = None;
    match operation {
        RootCorruptionOperation::CoveredByteFlip { offset, mask }
        | RootCorruptionOperation::ChecksumFieldFlip { offset, mask } => {
            after[*offset as usize] ^= mask;
            write_target(target_path, &after)?;
        }
        RootCorruptionOperation::FramingLengthLie {
            encoded_payload_length,
        } => {
            after[24..28].copy_from_slice(&encoded_payload_length.to_le_bytes());
            refresh_checksum(&mut after);
            write_target(target_path, &after)?;
        }
        RootCorruptionOperation::ScopeSubstitution { source_path } => {
            let donor_path = row_root.join(source_path);
            let donor = std::fs::read(&donor_path).map_err(|_| EditorAuditDenial::TargetRead)?;
            if Sha256::digest(&donor).as_slice() != record.substitution_source_sha256() {
                return Err(EditorAuditDenial::SourceChanged);
            }
            write_target(target_path, &donor)?;
            after = donor.clone();
            source = Some(donor);
        }
        RootCorruptionOperation::PointerCorruption { range, replacement } => {
            if range.end - range.start != 8 {
                return Err(EditorAuditDenial::ContractViolation);
            }
            after[range.start as usize..range.end as usize]
                .copy_from_slice(&replacement.to_le_bytes());
            refresh_checksum(&mut after);
            write_target(target_path, &after)?;
        }
        RootCorruptionOperation::StrictPrefixTruncation { retained_length } => {
            after.truncate(*retained_length as usize);
            write_target(target_path, &after)?;
        }
        RootCorruptionOperation::ArtifactRemoval => {
            std::fs::remove_file(target_path).map_err(|_| EditorAuditDenial::NamespaceMutation)?;
            removed_path = Some(record.relative_path().to_path_buf());
            return Ok(AppliedEdit::new(None, source, created_path, removed_path));
        }
        RootCorruptionOperation::ArtifactDuplication { destination } => {
            let destination_path = row_root.join(destination);
            if destination_path.exists() {
                return Err(EditorAuditDenial::NamespaceMutation);
            }
            ensure_parent(&destination_path)?;
            std::fs::write(&destination_path, before)
                .map_err(|_| EditorAuditDenial::TargetWrite)?;
            created_path = Some(destination.clone());
        }
        RootCorruptionOperation::UnsupportedFormatVersion { range, value } => {
            if range.end - range.start != 2 {
                return Err(EditorAuditDenial::ContractViolation);
            }
            after[range.start as usize..range.end as usize].copy_from_slice(&value.to_le_bytes());
            refresh_checksum(&mut after);
            write_target(target_path, &after)?;
        }
    }
    Ok(AppliedEdit::new(
        Some(after),
        source,
        created_path,
        removed_path,
    ))
}

fn write_target(path: &Path, bytes: &[u8]) -> Result<(), EditorAuditDenial> {
    std::fs::write(path, bytes).map_err(|_| EditorAuditDenial::TargetWrite)
}

fn ensure_parent(path: &Path) -> Result<(), EditorAuditDenial> {
    let parent = path.parent().ok_or(EditorAuditDenial::NamespaceMutation)?;
    std::fs::create_dir_all(parent).map_err(|_| EditorAuditDenial::NamespaceMutation)
}

const fn edit_refreshes_checksum(operation: &RootCorruptionOperation) -> u64 {
    match operation {
        RootCorruptionOperation::FramingLengthLie { .. }
        | RootCorruptionOperation::PointerCorruption { .. }
        | RootCorruptionOperation::UnsupportedFormatVersion { .. } => 1,
        _ => 0,
    }
}

const fn edit_bytes_read(operation: &RootCorruptionOperation, length: u64) -> u64 {
    match operation {
        RootCorruptionOperation::ScopeSubstitution { .. } => length * 2,
        _ => length,
    }
}

const fn edit_bytes_written(operation: &RootCorruptionOperation, length: u64) -> u64 {
    match operation {
        RootCorruptionOperation::ArtifactRemoval => 0,
        RootCorruptionOperation::StrictPrefixTruncation { retained_length } => *retained_length,
        _ => length,
    }
}
