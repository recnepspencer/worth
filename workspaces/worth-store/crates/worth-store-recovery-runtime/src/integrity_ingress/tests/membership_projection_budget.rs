use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    durable_artifact_checksum, PhysicalFreeSpaceMembershipBlock, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordFormatDeclaration,
    PhysicalSegmentId, PhysicalSegmentMembershipBlock, PhysicalTreeIdentity, RecordAllocationClass,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};

use super::super::projection::{
    free_space_membership_block, segment_membership_block, MembershipProjectionFailure,
};
use super::super::{RecoveryIntegrityIngressRejection, RecoveryIntegrityIngressTrace};

#[test]
fn segment_membership_projection_bounds_leaf_entries_and_branch_children() {
    let root = tempfile::tempdir().unwrap();
    initialize_media(root.path());
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(root.path())
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(4, 16_384).unwrap();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let tree = PhysicalTreeIdentity::new(71).unwrap();
    for block in segment_nodes(format) {
        let bytes = block.encode(format);
        let reference = block.reference(durable_artifact_checksum(&bytes));
        let artifact = RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = write_node(root.path(), "segment-manifests", artifact, &bytes);
        let source = discovery
            .read_segment_membership_block(reference.generation(), reference.block(), 4096)
            .unwrap();
        for (remaining, capacity) in [(0, 8), (1, 8), (2, 8), (1, 1)] {
            let mut trace = RecoveryIntegrityIngressTrace::default();
            let result = segment_membership_block(
                &source,
                discovery.store_identity(),
                format,
                tree,
                reference,
                capacity,
                remaining,
                &mut trace,
            );
            if capacity < 2 {
                assert_eq!(
                    result,
                    Err(MembershipProjectionFailure::Integrity(
                        RecoveryIntegrityIngressRejection::NonCanonicalEncoding
                    ))
                );
            } else if remaining < 2 {
                assert_eq!(
                    result,
                    Err(MembershipProjectionFailure::EntryLimit { observed: 2 })
                );
            } else {
                assert_eq!(result.unwrap(), block);
            }
            assert_eq!(
                (trace.counters().attempted, trace.counters().admitted),
                (1, 1)
            );
        }
        let mut damaged = bytes;
        damaged[44] ^= 1;
        std::fs::write(path, damaged).unwrap();
        let source = discovery
            .read_segment_membership_block(reference.generation(), reference.block(), 4096)
            .unwrap();
        let mut trace = RecoveryIntegrityIngressTrace::default();
        assert!(matches!(
            segment_membership_block(
                &source,
                discovery.store_identity(),
                format,
                tree,
                reference,
                8,
                1,
                &mut trace,
            ),
            Err(MembershipProjectionFailure::Integrity(
                RecoveryIntegrityIngressRejection::Integrity(_)
            ))
        ));
        assert_eq!(trace.counters().owner_projection_entries, 0);
    }
    drop(discovery.finish());
}

#[test]
fn free_space_projection_bounds_leaf_entries_and_branch_children() {
    let root = tempfile::tempdir().unwrap();
    initialize_media(root.path());
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(root.path())
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(4, 16_384).unwrap();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let tree = PhysicalTreeIdentity::new(71).unwrap();
    for block in free_space_nodes(format) {
        let bytes = block.encode(format);
        let reference = block.reference(durable_artifact_checksum(&bytes));
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = write_node(root.path(), "free-space", artifact, &bytes);
        let source = discovery
            .read_free_space_membership_block(reference.generation(), reference.block(), 4096)
            .unwrap();
        for (remaining, capacity) in [(0, 8), (1, 8), (2, 8), (1, 1)] {
            let mut trace = RecoveryIntegrityIngressTrace::default();
            let result = free_space_membership_block(
                &source,
                discovery.store_identity(),
                format,
                tree,
                reference,
                capacity,
                remaining,
                &mut trace,
            );
            if capacity < 2 {
                assert_eq!(
                    result,
                    Err(MembershipProjectionFailure::Integrity(
                        RecoveryIntegrityIngressRejection::NonCanonicalEncoding
                    ))
                );
            } else if remaining < 2 {
                assert_eq!(
                    result,
                    Err(MembershipProjectionFailure::EntryLimit { observed: 2 })
                );
            } else {
                assert_eq!(result.unwrap(), block);
            }
            assert_eq!(
                (trace.counters().attempted, trace.counters().admitted),
                (1, 1)
            );
        }
        let mut damaged = bytes;
        damaged[44] ^= 1;
        std::fs::write(path, damaged).unwrap();
        let source = discovery
            .read_free_space_membership_block(reference.generation(), reference.block(), 4096)
            .unwrap();
        let mut trace = RecoveryIntegrityIngressTrace::default();
        assert!(matches!(
            free_space_membership_block(
                &source,
                discovery.store_identity(),
                format,
                tree,
                reference,
                8,
                1,
                &mut trace,
            ),
            Err(MembershipProjectionFailure::Integrity(
                RecoveryIntegrityIngressRejection::Integrity(_)
            ))
        ));
        assert_eq!(trace.counters().owner_projection_entries, 0);
    }
    drop(discovery.finish());
}

fn segment_nodes(format: PhysicalRecordFormatDeclaration) -> [PhysicalSegmentMembershipBlock; 2] {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let entries = (1..=2)
        .map(|page| {
            RecordSegmentPageManifestEntry::new(
                authority
                    .page_cell(segment, PhysicalPageId::from_raw(page).unwrap())
                    .with_page_generation(generation),
                authority
                    .segment_cell(segment)
                    .with_segment_generation(generation),
                2,
                page as u32 - 1,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let children = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let child =
                PhysicalSegmentMembershipBlock::leaf(71, 1, index as u64 + 3, vec![*entry], 8)
                    .unwrap();
            child.reference(durable_artifact_checksum(&child.encode(format)))
        })
        .collect();
    [
        PhysicalSegmentMembershipBlock::leaf(71, 1, 1, entries, 8).unwrap(),
        PhysicalSegmentMembershipBlock::branch(71, 1, 2, 1, children, 8).unwrap(),
    ]
}

fn free_space_nodes(
    format: PhysicalRecordFormatDeclaration,
) -> [PhysicalFreeSpaceMembershipBlock; 2] {
    let entries = (1..=2)
        .map(|owner| {
            RecordFreeSpaceManifestEntry::new(RecordAllocationClass::InlinePage, owner, owner, 1, 8)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let children = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let child =
                PhysicalFreeSpaceMembershipBlock::leaf(71, 1, index as u64 + 3, vec![*entry], 8)
                    .unwrap();
            child.reference(durable_artifact_checksum(&child.encode(format)))
        })
        .collect();
    [
        PhysicalFreeSpaceMembershipBlock::leaf(71, 1, 1, entries, 8).unwrap(),
        PhysicalFreeSpaceMembershipBlock::branch(71, 1, 2, 1, children, 8).unwrap(),
    ]
}

fn write_node(
    root: &std::path::Path,
    family: &str,
    artifact: RecordArtifactFile,
    bytes: &[u8],
) -> std::path::PathBuf {
    let directory = root.join("families/records").join(family);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(artifact.file_name());
    std::fs::write(&path, bytes).unwrap();
    path
}

fn initialize_media(root: &std::path::Path) {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_path_buf()).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let TransitionOutcome::Success(media) =
        runtime.try_admit_filesystem_media(admission).into_raw()
    else {
        panic!("membership projection test requires real C4 media admission")
    };
    media.close();
}
