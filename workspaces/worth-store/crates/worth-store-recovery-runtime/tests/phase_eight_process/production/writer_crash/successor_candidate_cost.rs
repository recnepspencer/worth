use std::collections::VecDeque;
use std::path::Path;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PhysicalFreeSpaceMembershipBlock, PhysicalRootRoutingBlock, PhysicalSegmentMembershipBlock,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidateCost {
    pub(super) reads: u64,
    pub(super) raw_bytes: u64,
    pub(super) peak_bytes: u64,
    pub(super) comparison_scratch_bytes: u64,
    pub(super) manifest_entries: u64,
    pub(super) partial_peaks: CandidatePartialPeaks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidatePartialPeaks {
    pub(super) root_routing: u64,
    pub(super) segment_membership: u64,
    pub(super) free_space: u64,
}

pub(super) fn candidate_cost(root: &Path, generation: u64) -> CandidateCost {
    let records = root.join("families/records");
    let root_path = records
        .join("roots")
        .join(RecordArtifactFile::RootManifest { generation }.file_name());
    let root_bytes = std::fs::read(root_path).expect("read candidate root for cost oracle");
    let (manifest, format) = DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX)
        .expect("decode candidate root for cost oracle");
    assert_eq!(manifest.generation(), generation);

    let mut reads = 1_u64;
    let mut raw_bytes = root_bytes.len() as u64;
    let mut retained_artifacts = 1_usize;
    let mut retained_bytes = root_bytes.len() as u64;
    let mut placements = 0_usize;
    let mut segment_entries = 0_usize;
    let mut free_entries = 0_usize;
    let mut manifest_entries = 0_u64;
    let mut largest_artifact = root_bytes.len() as u64;
    let root_routing_peak = materialized_bytes(
        retained_bytes,
        retained_artifacts,
        2,
        placements,
        segment_entries,
        free_entries,
        1,
        0,
    );
    let mut retained_references = 1_usize;

    let mut root_queue = manifest.routing_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = root_queue.pop_front() {
        retained_references += 1;
        let bytes = std::fs::read(
            records.join("roots").join(
                RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate routing block for cost oracle");
        let (block, found_format) =
            PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate routing block for cost oracle");
        assert_eq!(found_format, format);
        placements += block.entries().map_or(0, <[_]>::len);
        manifest_entries += block
            .entries()
            .map_or_else(|| block.children().unwrap_or_default().len(), <[_]>::len)
            as u64;
        root_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
            largest_artifact = largest_artifact.max(bytes.len() as u64);
        }
    }

    let segment_membership_peak = materialized_bytes(
        retained_bytes,
        retained_artifacts,
        retained_references + 1,
        placements,
        segment_entries,
        free_entries,
        1,
        0,
    );

    let mut segment_queue = manifest.segment_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = segment_queue.pop_front() {
        retained_references += 1;
        let bytes = std::fs::read(
            records.join("segment-manifests").join(
                RecordArtifactFile::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate segment block for cost oracle");
        let (block, found_format) =
            PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate segment block for cost oracle");
        assert_eq!(found_format, format);
        segment_entries += block.entries().map_or(0, <[_]>::len);
        manifest_entries += block
            .entries()
            .map_or_else(|| block.children().unwrap_or_default().len(), <[_]>::len)
            as u64;
        segment_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
            largest_artifact = largest_artifact.max(bytes.len() as u64);
        }
    }

    let free_bytes = std::fs::read(
        records
            .join("free-space")
            .join(RecordArtifactFile::FreeSpaceManifest { generation }.file_name()),
    )
    .expect("read candidate free-space header for cost oracle");
    let (free, found_format) =
        DurableFreeSpaceManifestHeader::decode(&free_bytes, manifest.node_capacity())
            .expect("decode candidate free-space header for cost oracle");
    assert_eq!(found_format, format);
    reads += 1;
    raw_bytes += free_bytes.len() as u64;
    retained_artifacts += 1;
    retained_bytes += free_bytes.len() as u64;
    largest_artifact = largest_artifact.max(free_bytes.len() as u64);
    retained_references += 1;

    let free_space_peak = materialized_bytes(
        retained_bytes,
        retained_artifacts,
        retained_references + 1,
        placements,
        segment_entries,
        free_entries,
        1,
        1,
    );

    let mut free_queue = free.root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = free_queue.pop_front() {
        retained_references += 1;
        let bytes = std::fs::read(
            records.join("free-space").join(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate free-space block for cost oracle");
        let (block, found_format) =
            PhysicalFreeSpaceMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate free-space block for cost oracle");
        assert_eq!(found_format, format);
        free_entries += block.entries().map_or(0, <[_]>::len);
        manifest_entries += block
            .entries()
            .map_or_else(|| block.children().unwrap_or_default().len(), <[_]>::len)
            as u64;
        free_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
            largest_artifact = largest_artifact.max(bytes.len() as u64);
        }
    }

    let peak_bytes = materialized_bytes(
        retained_bytes,
        retained_artifacts,
        retained_references,
        placements,
        segment_entries,
        free_entries,
        1,
        1,
    );
    CandidateCost {
        reads,
        raw_bytes,
        peak_bytes,
        comparison_scratch_bytes: largest_artifact,
        manifest_entries,
        partial_peaks: CandidatePartialPeaks {
            root_routing: root_routing_peak,
            segment_membership: segment_membership_peak,
            free_space: free_space_peak,
        },
    }
}

fn materialized_bytes(
    retained_bytes: u64,
    retained_artifacts: usize,
    retained_references: usize,
    placements: usize,
    segment_entries: usize,
    free_entries: usize,
    roots: usize,
    free_headers: usize,
) -> u64 {
    retained_bytes
        + (roots * std::mem::size_of::<DurablePhysicalRootManifest>()) as u64
        + (free_headers * std::mem::size_of::<DurableFreeSpaceManifestHeader>()) as u64
        + (placements * std::mem::size_of::<CurrentPhysicalRecordPlacement>()) as u64
        + (segment_entries * std::mem::size_of::<RecordSegmentPageManifestEntry>()) as u64
        + (free_entries * std::mem::size_of::<RecordFreeSpaceManifestEntry>()) as u64
        + (retained_artifacts
            * (std::mem::size_of::<RecordArtifactFile>() + std::mem::size_of::<Box<[u8]>>()))
            as u64
        + (retained_references * std::mem::size_of::<RecordArtifactFile>()) as u64
}
