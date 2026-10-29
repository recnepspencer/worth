pub(crate) use {std::collections::VecDeque};
pub(crate) use {std::path::Path};

pub(crate) use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PhysicalFreeSpaceMembershipBlock, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFreeSpaceManifestEntry, worth_store_physical_format::RecordSegmentPageManifestEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CandidateCost {
    pub(crate) reads: u64,
    pub(crate) raw_bytes: u64,
    pub(crate) peak_bytes: u64,
    pub(crate) manifest_entries: u64,
    pub(crate) partial_peaks: CandidatePartialPeaks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CandidatePartialPeaks {
    pub(crate) root_routing: u64,
    pub(crate) segment_membership: u64,
    pub(crate) free_space: u64,
}

pub(crate) fn candidate_cost(root: &Path, generation: u64) -> CandidateCost {
    let records = root.join("families/records");
    let root_path = records
        .join("roots")
        .join(worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name());
    let root_bytes = std::fs::read(root_path).expect("read candidate root for cost oracle");
    let (manifest, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX)
        .expect("decode candidate root for cost oracle");
    assert_eq!(manifest.generation(), generation);

    let mut reads = 1_u64;
    let mut raw_bytes = root_bytes.len() as u64;
    let mut retained_artifacts = 1_usize;
    let mut retained_bytes = root_bytes.len() as u64;
    let mut placements = 0_usize;
    let mut segment_entries = 0_usize;
    let mut free_entries = 0_usize;
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
                worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate routing block for cost oracle");
        let (block, found_format) =
            worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate routing block for cost oracle");
        assert_eq!(found_format, format);
        placements += block.entries().map_or(0, <[_]>::len);
        root_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
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
                worth_store_physical_format::RecordArtifactFile::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate segment block for cost oracle");
        let (block, found_format) =
            worth_store_physical_format::PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate segment block for cost oracle");
        assert_eq!(found_format, format);
        segment_entries += block.entries().map_or(0, <[_]>::len);
        segment_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
        }
    }

    let free_bytes = std::fs::read(
        records
            .join("free-space")
            .join(worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest { generation }.file_name()),
    )
    .expect("read candidate free-space header for cost oracle");
    let (free, found_format) =
        worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&free_bytes, manifest.node_capacity())
            .expect("decode candidate free-space header for cost oracle");
    assert_eq!(found_format, format);
    reads += 1;
    raw_bytes += free_bytes.len() as u64;
    retained_artifacts += 1;
    retained_bytes += free_bytes.len() as u64;
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
                worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read candidate free-space block for cost oracle");
        let (block, found_format) =
            worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode candidate free-space block for cost oracle");
        assert_eq!(found_format, format);
        free_entries += block.entries().map_or(0, <[_]>::len);
        free_queue.extend(block.children().unwrap_or_default().iter().copied());
        reads += 1;
        raw_bytes += bytes.len() as u64;
        if reference.generation() == generation {
            retained_artifacts += 1;
            retained_bytes += bytes.len() as u64;
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
        // The candidate root's one entry and its leaf entries: blocks and
        // branch children charge nothing.
        manifest_entries: (1 + placements + segment_entries + free_entries) as u64,
        partial_peaks: CandidatePartialPeaks {
            root_routing: root_routing_peak,
            segment_membership: segment_membership_peak,
            free_space: free_space_peak,
        },
    }
}

pub(crate) fn materialized_bytes(
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