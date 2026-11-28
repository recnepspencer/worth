//! Paged per-basis manifest (`WHBM4`) and out-of-line continuation reference
//! tree (`WHCM1`) publication, loading, and offline chunk consolidation I/O.
//!
//! Continuation reference trees across ALL chunks are published **batched
//! level-by-level** so total member-set publications scale strictly with
//! `O(tree_height)` (`<= 3`), NEVER `O(N_chunks)`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::HistoricalReadDenial;
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, PhysicalRecordReader, ServingPhysicalRuntime,
};

use super::basis_manifest::{
    collect_all_manifest_chunks, partition_manifest_index, partition_manifest_leaves,
    BasisManifestChildRef, ManifestIndexPageMeta,
};
use super::catalog::RetainedBasisRecord;
use super::chunk_format::{
    compute_root_semantic_commitment, member_id, ChunkMemberId, SealedChunkDescriptor,
};
use super::chunk_scan::consolidate_chunk;
use super::continuation_manifest::{
    collect_all_continuation_refs, partition_continuation_index, partition_continuation_leaves,
    ContIndexChild, ContRefEntry,
};
use super::durable_io::{
    basis_stage_anchor_name, corrupt, encode_basis_header, publish_anchor, DurableCatalogState,
    DurableMemberReader,
};
use super::residency::HistoricalWorkingSetPool;
use super::stream::LoanedMemberReader;

fn index_locators(
    members: &[AnchoredRecordMemberLocator],
    locators: &mut BTreeMap<ChunkMemberId, [u8; 64]>,
) {
    for loc in members {
        locators.insert(
            ChunkMemberId {
                sha256: loc.sha256(),
                length: loc.length(),
            },
            loc.to_bytes(),
        );
    }
}

fn publish_stage_pages<K>(
    carried: &mut Vec<AnchoredRecordMemberLocator>,
    locators: &mut BTreeMap<ChunkMemberId, [u8; 64]>,
    level_pages: Vec<(K, Vec<u8>)>,
    publish_stage: &mut impl FnMut(
        Vec<AnchoredRecordMember>,
    )
        -> Result<Vec<AnchoredRecordMemberLocator>, HistoricalReadDenial>,
) -> Result<Vec<(K, ChunkMemberId)>, HistoricalReadDenial> {
    let mut stage: Vec<_> = carried
        .iter()
        .copied()
        .map(|locator| AnchoredRecordMember::Carried { locator })
        .collect();
    let mut pending = Vec::with_capacity(level_pages.len());
    let mut staged_keys = BTreeSet::new();
    for (key, page_bytes) in level_pages {
        let id = member_id(&page_bytes);
        if !locators.contains_key(&id) && staged_keys.insert(id) {
            stage.push(AnchoredRecordMember::New {
                key: id.sha256,
                bytes: page_bytes,
            });
        }
        pending.push((key, id));
    }
    *carried = publish_stage(stage)?;
    index_locators(carried, locators);
    Ok(pending)
}

/// Publishes ALL chunks' out-of-line `WHCM1` continuation reference trees
/// batched level-by-level (1 anchor publication per tree height across ALL
/// chunks, strictly `O(tree_height)` instead of `O(N_chunks)`).
fn publish_continuation_trees_batched(
    chunks: &mut [SealedChunkDescriptor],
    carried: &mut Vec<AnchoredRecordMemberLocator>,
    locators: &mut BTreeMap<ChunkMemberId, [u8; 64]>,
    publish_stage: &mut impl FnMut(
        Vec<AnchoredRecordMember>,
    )
        -> Result<Vec<AnchoredRecordMemberLocator>, HistoricalReadDenial>,
) -> Result<(), HistoricalReadDenial> {
    let mut active: Vec<(usize, Vec<(ChunkMemberId, Vec<u8>)>)> = Vec::new();
    for (chunk_idx, chunk) in chunks.iter_mut().enumerate() {
        chunk.chunk_locator = Some(*locators.get(&chunk.chunk_member).ok_or_else(corrupt)?);
        chunk.continuation_locators = chunk
            .continuation_members
            .iter()
            .map(|id| locators.get(id).copied().ok_or_else(corrupt))
            .collect::<Result<Vec<_>, _>>()?;
        chunk.continuation_count = chunk.continuation_members.len() as u32;
        if chunk.continuation_members.is_empty() {
            chunk.continuation_manifest_member = None;
            chunk.continuation_manifest_locator = None;
        } else {
            let mut entries: Vec<ContRefEntry> = chunk
                .continuation_members
                .iter()
                .zip(chunk.continuation_locators.iter())
                .map(|(&member, &locator)| ContRefEntry { member, locator })
                .collect();
            entries.sort_unstable_by_key(|e| e.member);
            entries.dedup_by_key(|e| e.member);
            active.push((chunk_idx, partition_continuation_leaves(&entries)?));
        }
    }

    while !active.is_empty() {
        let mut prev_counts: BTreeMap<usize, usize> = BTreeMap::new();
        let mut batch_pages: Vec<((usize, ChunkMemberId), Vec<u8>)> = Vec::new();
        for (chunk_idx, pages) in active {
            prev_counts.insert(chunk_idx, pages.len());
            for (max_member, bytes) in pages {
                batch_pages.push(((chunk_idx, max_member), bytes));
            }
        }
        let published = publish_stage_pages(carried, locators, batch_pages, publish_stage)?;
        let mut grouped: BTreeMap<usize, Vec<(ChunkMemberId, ChunkMemberId)>> = BTreeMap::new();
        for ((chunk_idx, max_member), page_id) in published {
            grouped
                .entry(chunk_idx)
                .or_default()
                .push((max_member, page_id));
        }
        let mut next_active = Vec::new();
        for (chunk_idx, mut level_ids) in grouped {
            if level_ids.len() == 1 {
                let (_, root_id) = level_ids.pop().expect("single continuation root");
                chunks[chunk_idx].continuation_manifest_member = Some(root_id);
                chunks[chunk_idx].continuation_manifest_locator =
                    Some(*locators.get(&root_id).ok_or_else(corrupt)?);
            } else {
                let children: Vec<_> = level_ids
                    .into_iter()
                    .map(|(max_member, member)| {
                        Ok(ContIndexChild {
                            max_member,
                            member,
                            locator: *locators.get(&member).ok_or_else(corrupt)?,
                        })
                    })
                    .collect::<Result<_, HistoricalReadDenial>>()?;
                let next_pages = partition_continuation_index(&children)?;
                let prev_len = prev_counts.get(&chunk_idx).copied().unwrap_or(0);
                if next_pages.len() >= prev_len {
                    return Err(corrupt());
                }
                next_active.push((chunk_idx, next_pages));
            }
        }
        active = next_active;
    }
    Ok(())
}

/// Publishes all chunk & continuation members, batched out-of-line bounded
/// `<= 3 KiB` per-chunk continuation reference trees (`WHCM1`), AND the paged
/// `WHBM4` manifest tree with explicit convergence verification on every level.
pub(super) fn publish_basis_with_manifest(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
    commit_id: CommitId,
    epoch: u64,
    root_commitment: &[u8; 32],
    chunks: &mut [SealedChunkDescriptor],
    new_members: &BTreeMap<ChunkMemberId, Vec<u8>>,
) -> Result<(Option<ChunkMemberId>, Option<[u8; 64]>), HistoricalReadDenial> {
    if chunks.is_empty() {
        return Ok((None, None));
    }
    let header = encode_basis_header(commit_id, epoch, root_commitment);
    let mut needed = BTreeSet::new();
    for chunk in chunks.iter() {
        needed.insert(chunk.chunk_member);
        needed.extend(chunk.continuation_members.iter().copied());
    }
    let members: Vec<_> = needed
        .iter()
        .map(|id| {
            Ok(AnchoredRecordMember::New {
                key: id.sha256,
                bytes: new_members.get(id).ok_or_else(corrupt)?.clone(),
            })
        })
        .collect::<Result<_, HistoricalReadDenial>>()?;
    let mut publish_stage = |m| {
        publish_anchor(
            serving,
            basis_stage_anchor_name(),
            &mut state.basis_stage_generation,
            header.clone(),
            m,
        )
    };
    let mut carried = publish_stage(members)?;
    let mut locators = BTreeMap::new();
    index_locators(&carried, &mut locators);

    publish_continuation_trees_batched(chunks, &mut carried, &mut locators, &mut publish_stage)?;

    let leaf_pages = partition_manifest_leaves(chunks)?;
    let mut prev_len = leaf_pages.len();
    let mut leaf_pending =
        publish_stage_pages(&mut carried, &mut locators, leaf_pages, &mut publish_stage)?;
    if leaf_pending.len() == 1 {
        let (_, root_id) = leaf_pending.pop().expect("single leaf root");
        return Ok((
            Some(root_id),
            Some(*locators.get(&root_id).ok_or_else(corrupt)?),
        ));
    }
    let mut children: Vec<BasisManifestChildRef> = leaf_pending
        .into_iter()
        .map(|(route_max, member)| {
            let locator = *locators.get(&member).ok_or_else(corrupt)?;
            Ok(BasisManifestChildRef {
                route_max,
                member,
                locator,
                rightmost_leaf_member: member,
                rightmost_leaf_locator: locator,
            })
        })
        .collect::<Result<_, HistoricalReadDenial>>()?;

    loop {
        let index_pages = partition_manifest_index(&children)?;
        // Explicit convergence guard: index publication must strictly shrink at every level.
        if index_pages.len() >= prev_len {
            return Err(corrupt());
        }
        prev_len = index_pages.len();
        let mut pending =
            publish_stage_pages(&mut carried, &mut locators, index_pages, &mut publish_stage)?;
        if pending.len() == 1 {
            let (_, root_id) = pending.pop().expect("single index root");
            return Ok((
                Some(root_id),
                Some(*locators.get(&root_id).ok_or_else(corrupt)?),
            ));
        }
        children = pending
            .into_iter()
            .map(
                |(
                    ManifestIndexPageMeta {
                        route_max,
                        rightmost_leaf_member,
                        rightmost_leaf_locator,
                    },
                    member,
                )| {
                    Ok(BasisManifestChildRef {
                        route_max,
                        member,
                        locator: *locators.get(&member).ok_or_else(corrupt)?,
                        rightmost_leaf_member,
                        rightmost_leaf_locator,
                    })
                },
            )
            .collect::<Result<_, HistoricalReadDenial>>()?;
    }
}

/// Walks and decodes ONE basis's paged `BasisManifest` tree on demand under
/// the isolated `Historical` residency pool.
pub(super) fn load_basis_manifest_on_demand(
    residency: &HistoricalWorkingSetPool,
    reader: &DurableMemberReader,
    entry: &RetainedBasisRecord,
) -> Result<Arc<[SealedChunkDescriptor]>, HistoricalReadDenial> {
    let Some(manifest_id) = entry.basis_manifest_member else {
        return Ok(Arc::from(Vec::new()));
    };
    collect_all_manifest_chunks(
        manifest_id,
        entry.basis_manifest_locator.as_ref(),
        |id, loc| residency.load_historical_manifest(id, || reader.read_member(id, loc)),
    )
}

pub(super) fn consolidate_durable_chunks(
    residency: &HistoricalWorkingSetPool,
    pinned_reader: PhysicalRecordReader,
    corrupted_overrides: Arc<RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
    chunks: &[SealedChunkDescriptor],
    expected_root: [u8; 32],
) -> Result<(Vec<SealedChunkDescriptor>, BTreeMap<ChunkMemberId, Vec<u8>>), HistoricalReadDenial> {
    let reader = DurableMemberReader::new(pinned_reader, corrupted_overrides);
    let mut new_chunks = Vec::with_capacity(chunks.len());
    let mut new_members = BTreeMap::new();
    for chunk in chunks {
        let mut hydrated = chunk.clone();
        let cont_map: BTreeMap<ChunkMemberId, [u8; 64]> = if let Some(cont_root) =
            chunk.continuation_manifest_member
        {
            let refs = collect_all_continuation_refs(
                cont_root,
                chunk.continuation_manifest_locator.as_ref(),
                |id, loc| residency.load_historical_manifest(id, || reader.read_member(id, loc)),
            )?;
            hydrated.continuation_members = refs.iter().map(|e| e.member).collect();
            hydrated.continuation_locators = refs.iter().map(|e| e.locator).collect();
            refs.into_iter().map(|e| (e.member, e.locator)).collect()
        } else {
            chunk
                .continuation_members
                .iter()
                .copied()
                .zip(chunk.continuation_locators.iter().copied())
                .collect()
        };
        let chunk_bytes =
            reader.read_member(hydrated.chunk_member, hydrated.chunk_locator.as_ref())?;
        let (mut updated, flat_bytes) = consolidate_chunk(&hydrated, &chunk_bytes, |cont_id| {
            let loc = cont_map.get(&cont_id);
            let bytes = reader.read_member(cont_id, loc)?;
            new_members.entry(cont_id).or_insert_with(|| bytes.clone());
            Ok(bytes)
        })?;
        updated.continuation_members = hydrated.continuation_members.clone();
        updated.continuation_count = updated.continuation_members.len() as u32;
        for &cont_id in &updated.continuation_members {
            if !new_members.contains_key(&cont_id) {
                let bytes = reader.read_member(cont_id, cont_map.get(&cont_id))?;
                new_members.insert(cont_id, bytes);
            }
        }
        new_members.insert(updated.chunk_member, flat_bytes);
        new_chunks.push(updated);
    }
    let comms: Vec<_> = new_chunks.iter().map(|c| c.semantic_commitment).collect();
    if compute_root_semantic_commitment(&comms) != expected_root {
        return Err(corrupt());
    }
    Ok((new_chunks, new_members))
}
