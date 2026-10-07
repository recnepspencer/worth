use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, FreeSpaceBlockReference,
    ManifestBlockReference, PhysicalFreeSpaceMembershipBlock, PhysicalRootRoutingBlock,
    PhysicalSegmentMembershipBlock, RecordArtifactFile, RecordFreeSpaceManifestEntry,
    RecordSegmentPageManifestEntry, SegmentManifestBlockReference,
};

use super::{encoding, CandidateBuild, CandidateBuildDenial};
use crate::progression::planned::PlanningResidentAllowance;

pub(super) fn root_routing(
    build: &mut CandidateBuild,
    entries: &[CurrentPhysicalRecordPlacement],
    tree: u64,
    generation: u64,
    capacity: u16,
    mut next_block: u64,
) -> Result<(Option<ManifestBlockReference>, u64), CandidateBuildDenial> {
    let width = usize::from(capacity);
    let mut roots = build
        .allowance
        .reserve::<ManifestBlockReference>(level_count(entries.len(), width)?)?;
    for chunk in entries.chunks(usize::from(capacity)) {
        let block_id = allocate(&mut next_block)?;
        let mut leaf = build
            .allowance
            .reserve::<CurrentPhysicalRecordPlacement>(chunk.len())?;
        leaf.extend_from_slice(chunk);
        let block =
            encoding::root_leaf(tree, generation, block_id, leaf, capacity, build.allowance)?;
        let bytes = encoding::root_block(&block, build.format, build.allowance)?;
        roots.push(block.reference(durable_artifact_checksum(&bytes)));
        let block_heap = block
            .owned_heap_bytes()
            .ok_or(CandidateBuildDenial::Invalid)?;
        drop(block);
        build.allowance.release(block_heap);
        build.push(
            RecordArtifactFile::RootRoutingBlock {
                generation,
                block: block_id,
            },
            bytes,
        )?;
    }
    while roots.len() > 1 {
        let mut parents = build
            .allowance
            .reserve::<ManifestBlockReference>(level_count(roots.len(), width)?)?;
        for chunk in roots.chunks(usize::from(capacity)) {
            let block_id = allocate(&mut next_block)?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut children = build
                .allowance
                .reserve::<ManifestBlockReference>(chunk.len())?;
            children.extend_from_slice(chunk);
            let block = PhysicalRootRoutingBlock::branch(
                tree, generation, block_id, level, children, capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::root_block(&block, build.format, build.allowance)?;
            parents.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_heap = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            build.allowance.release(block_heap);
            build.push(
                RecordArtifactFile::RootRoutingBlock {
                    generation,
                    block: block_id,
                },
                bytes,
            )?;
        }
        release_backing(roots, build.allowance)?;
        roots = parents;
    }
    let root = roots.pop();
    release_backing(roots, build.allowance)?;
    Ok((root, next_block))
}

pub(super) fn segment_routing(
    build: &mut CandidateBuild,
    entries: &[RecordSegmentPageManifestEntry],
    tree: u64,
    generation: u64,
    capacity: u16,
    mut next_block: u64,
) -> Result<(Option<SegmentManifestBlockReference>, u64), CandidateBuildDenial> {
    let width = usize::from(capacity);
    let mut roots = build
        .allowance
        .reserve::<SegmentManifestBlockReference>(level_count(entries.len(), width)?)?;
    for chunk in entries.chunks(usize::from(capacity)) {
        let block_id = allocate(&mut next_block)?;
        let mut leaf = build
            .allowance
            .reserve::<RecordSegmentPageManifestEntry>(chunk.len())?;
        leaf.extend_from_slice(chunk);
        let block =
            PhysicalSegmentMembershipBlock::leaf(tree, generation, block_id, leaf, capacity)
                .ok_or(CandidateBuildDenial::Invalid)?;
        let bytes = encoding::segment_block(&block, build.format, build.allowance)?;
        roots.push(block.reference(durable_artifact_checksum(&bytes)));
        let block_heap = block
            .owned_heap_bytes()
            .ok_or(CandidateBuildDenial::Invalid)?;
        drop(block);
        build.allowance.release(block_heap);
        build.push(
            RecordArtifactFile::SegmentMembershipBlock {
                generation,
                block: block_id,
            },
            bytes,
        )?;
    }
    while roots.len() > 1 {
        let mut parents = build
            .allowance
            .reserve::<SegmentManifestBlockReference>(level_count(roots.len(), width)?)?;
        for chunk in roots.chunks(usize::from(capacity)) {
            let block_id = allocate(&mut next_block)?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut children = build
                .allowance
                .reserve::<SegmentManifestBlockReference>(chunk.len())?;
            children.extend_from_slice(chunk);
            let block = PhysicalSegmentMembershipBlock::branch(
                tree, generation, block_id, level, children, capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::segment_block(&block, build.format, build.allowance)?;
            parents.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_heap = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            build.allowance.release(block_heap);
            build.push(
                RecordArtifactFile::SegmentMembershipBlock {
                    generation,
                    block: block_id,
                },
                bytes,
            )?;
        }
        release_backing(roots, build.allowance)?;
        roots = parents;
    }
    let root = roots.pop();
    release_backing(roots, build.allowance)?;
    Ok((root, next_block))
}

pub(super) fn free_space_routing(
    build: &mut CandidateBuild,
    entries: &[RecordFreeSpaceManifestEntry],
    tree: u64,
    generation: u64,
    capacity: u16,
    mut next_block: u64,
) -> Result<(Option<FreeSpaceBlockReference>, u64), CandidateBuildDenial> {
    let width = usize::from(capacity);
    let mut roots = build
        .allowance
        .reserve::<FreeSpaceBlockReference>(level_count(entries.len(), width)?)?;
    for chunk in entries.chunks(usize::from(capacity)) {
        let block_id = allocate(&mut next_block)?;
        let mut leaf = build
            .allowance
            .reserve::<RecordFreeSpaceManifestEntry>(chunk.len())?;
        leaf.extend_from_slice(chunk);
        let block =
            PhysicalFreeSpaceMembershipBlock::leaf(tree, generation, block_id, leaf, capacity)
                .ok_or(CandidateBuildDenial::Invalid)?;
        let bytes = encoding::free_block(&block, build.format, build.allowance)?;
        roots.push(block.reference(durable_artifact_checksum(&bytes)));
        let block_heap = block
            .owned_heap_bytes()
            .ok_or(CandidateBuildDenial::Invalid)?;
        drop(block);
        build.allowance.release(block_heap);
        build.push(
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation,
                block: block_id,
            },
            bytes,
        )?;
    }
    while roots.len() > 1 {
        let mut parents = build
            .allowance
            .reserve::<FreeSpaceBlockReference>(level_count(roots.len(), width)?)?;
        for chunk in roots.chunks(usize::from(capacity)) {
            let block_id = allocate(&mut next_block)?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut children = build
                .allowance
                .reserve::<FreeSpaceBlockReference>(chunk.len())?;
            children.extend_from_slice(chunk);
            let block = PhysicalFreeSpaceMembershipBlock::branch(
                tree, generation, block_id, level, children, capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::free_block(&block, build.format, build.allowance)?;
            parents.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_heap = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            build.allowance.release(block_heap);
            build.push(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation,
                    block: block_id,
                },
                bytes,
            )?;
        }
        release_backing(roots, build.allowance)?;
        roots = parents;
    }
    let root = roots.pop();
    release_backing(roots, build.allowance)?;
    Ok((root, next_block))
}

fn allocate(next: &mut u64) -> Result<u64, CandidateBuildDenial> {
    let block = *next;
    *next = next.checked_add(1).ok_or(CandidateBuildDenial::Invalid)?;
    if block == 0 {
        return Err(CandidateBuildDenial::Invalid);
    }
    Ok(block)
}

fn level_count(entries: usize, capacity: usize) -> Result<usize, CandidateBuildDenial> {
    if capacity < 2 {
        return Err(CandidateBuildDenial::Invalid);
    }
    entries
        .checked_add(capacity - 1)
        .map(|count| count / capacity)
        .ok_or(CandidateBuildDenial::Invalid)
}

fn release_backing<T>(
    values: Vec<T>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(), CandidateBuildDenial> {
    let bytes = PlanningResidentAllowance::vector_bytes(&values)?;
    drop(values);
    allowance.release(bytes);
    Ok(())
}
