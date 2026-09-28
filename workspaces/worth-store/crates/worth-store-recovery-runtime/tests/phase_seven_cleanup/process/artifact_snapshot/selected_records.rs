use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalFreeSpaceMembershipBlock,
    PhysicalRootRoutingBlock, PhysicalSegmentMembershipBlock, RecordArtifactFile,
};

use super::{collect_file, record_artifact_path};

pub(super) fn capture(root: &Path, paths: &mut BTreeSet<PathBuf>) {
    collect_file(
        &record_artifact_path(root, RecordArtifactFile::BootstrapCatalog),
        paths,
    );
    let current = read_selector(root, RecordArtifactFile::CurrentRootSelector, paths);
    collect_selected_root(root, current, paths);
    if record_artifact_path(root, RecordArtifactFile::PreviousRootSelector).is_file() {
        let previous = read_selector(root, RecordArtifactFile::PreviousRootSelector, paths);
        collect_selected_root(root, previous, paths);
    }
}

fn read_selector(
    root: &Path,
    artifact: RecordArtifactFile,
    paths: &mut BTreeSet<PathBuf>,
) -> DurableRootSelector {
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    DurableRootSelector::decode(&std::fs::read(path).expect("read selected root selector"))
        .expect("decode selected root selector")
}

fn collect_selected_root(
    root: &Path,
    selector: DurableRootSelector,
    paths: &mut BTreeSet<PathBuf>,
) {
    let artifact = RecordArtifactFile::RootManifest {
        generation: selector.root_generation(),
    };
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    let (manifest, format) = DurablePhysicalRootManifest::decode(
        &std::fs::read(path).expect("read selected root manifest"),
        u16::MAX,
    )
    .expect("decode selected root manifest");
    assert_eq!(format, selector.format());
    collect_root_tree(root, &manifest, format, paths);
    collect_segment_tree(root, &manifest, format, paths);
    collect_free_space_tree(root, &manifest, format, paths);
}

fn collect_root_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let mut pending = manifest.routing_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected root routing block");
        let (block, found_format) =
            PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode selected root routing block");
        assert_eq!(found_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                for placement in entries {
                    collect_placement(root, placement, paths);
                }
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
}

fn collect_placement(
    root: &Path,
    placement: CurrentPhysicalRecordPlacement,
    paths: &mut BTreeSet<PathBuf>,
) {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(inline) => collect_file(
            &record_artifact_path(
                root,
                RecordArtifactFile::Segment {
                    segment: inline.segment().get(),
                    generation: inline.segment_generation(),
                },
            ),
            paths,
        ),
        CurrentPhysicalRecordPlacement::Extent(extent) => {
            collect_file(
                &record_artifact_path(
                    root,
                    RecordArtifactFile::ExtentArena {
                        arena: extent.arena_range().arena().get(),
                    },
                ),
                paths,
            );
        }
    }
}

fn collect_segment_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let mut pending = manifest.segment_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected segment membership block");
        let (block, found_format) =
            PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode selected segment membership block");
        assert_eq!(found_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        if let Some(children) = block.children() {
            pending.extend(children.iter().copied());
        }
    }
}

fn collect_free_space_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let artifact = RecordArtifactFile::FreeSpaceManifest {
        generation: manifest.generation(),
    };
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    let bytes = std::fs::read(path).expect("read selected free-space manifest");
    let (header, found_format) = DurableFreeSpaceManifestHeader::decode(&bytes, u16::MAX)
        .expect("decode selected free-space manifest");
    assert_eq!(found_format, format);
    assert_eq!(
        durable_artifact_checksum(&bytes),
        manifest.free_space_checksum()
    );
    let mut pending = header.root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected free-space membership block");
        let (block, block_format) =
            PhysicalFreeSpaceMembershipBlock::decode(&bytes, header.node_capacity())
                .expect("decode selected free-space membership block");
        assert_eq!(block_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        if let Some(children) = block.children() {
            pending.extend(children.iter().copied());
        }
    }
}
