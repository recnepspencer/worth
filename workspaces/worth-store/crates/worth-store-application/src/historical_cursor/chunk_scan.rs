//! Zero-copy bounded incremental chunk scanning, preflight continuation-size
//! enforcement, per-slot continuation verification, and history layer
//! consolidation (Store Milestone 4, Phase M4.4).

use std::cmp::Ordering;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_runtime_world::facade::{
    AdmittedHistoricalProjection, CanonicalCursorKey, CanonicalKeyDomain, HistoricalBasisRow,
    HistoricalQueryContract, HistoricalReadDenial,
};

use super::chunk_codec::{
    decode_borrowed_chunk_rows, decode_borrowed_continuation_page, decode_chunk_image,
    encode_chunk_image, visit_continuation_ids, visit_length_prefixed_slices,
};
use super::chunk_format::{
    compute_chunk_semantic_commitment_from_borrowed,
    compute_chunk_semantic_commitment_from_physical, corrupt, member_id, verify_member_digest,
    BorrowedChunkRow, ChunkMemberId, PhysicalChunkImage, PhysicalChunkRow, SealedChunkDescriptor,
    ROW_HISTORY_DOMAIN,
};

fn row_is_strictly_after(
    partition_id: u32,
    domain: CanonicalKeyDomain,
    logical_key: &[u8],
    after: &CanonicalCursorKey,
) -> bool {
    match after {
        CanonicalCursorKey::Start => true,
        CanonicalCursorKey::After {
            partition_id: after_part,
            domain: after_dom,
            logical_key: after_key,
        } => {
            (partition_id, domain, logical_key).cmp(&(
                *after_part,
                *after_dom,
                after_key.as_slice(),
            )) == Ordering::Greater
        }
    }
}

/// Incrementally scans one chunk strictly after `after` using zero-copy borrowed
/// slices over `chunk_bytes`, verifying the chunk commitment and enforcing
/// `streaming_window_bytes` BEFORE owning any payload/history buffers or
/// faulting continuation pages.
#[allow(clippy::too_many_arguments)]
pub(crate) fn scan_verified_chunk_bounded<F>(
    descriptor: &SealedChunkDescriptor,
    chunk_bytes: &[u8],
    after: &CanonicalCursorKey,
    query: &HistoricalQueryContract,
    projection: AdmittedHistoricalProjection,
    limit: usize,
    streaming_window_bytes: usize,
    accumulated_bytes: &mut usize,
    collected: &mut Vec<(CanonicalCursorKey, HistoricalBasisRow)>,
    mut read_continuation: F,
) -> Result<bool, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    verify_member_digest(descriptor.chunk_member, chunk_bytes)?;
    let mut ordered = decode_borrowed_chunk_rows(chunk_bytes)?;
    ordered.sort_by(|a, b| {
        (a.partition_id, a.domain, a.logical_key).cmp(&(b.partition_id, b.domain, b.logical_key))
    });
    if compute_chunk_semantic_commitment_from_borrowed(&ordered) != descriptor.semantic_commitment {
        return Err(corrupt());
    }

    let window_u64 = streaming_window_bytes as u64;
    for physical in ordered {
        if physical.domain != query.domain() {
            continue;
        }
        if let Some(kind) = query.kind_filter()
            && KindId(physical.kind_id) != kind
        {
            continue;
        }
        if !row_is_strictly_after(
            physical.partition_id,
            physical.domain,
            physical.logical_key,
            after,
        ) {
            continue;
        }
        if collected.len() >= limit {
            return Ok(true);
        }

        let base_bytes = physical.logical_key.len().saturating_add(match projection {
            AdmittedHistoricalProjection::KeysOnly => 0,
            AdmittedHistoricalProjection::HeadOnly | AdmittedHistoricalProjection::Full => {
                physical.payload.len()
            }
            AdmittedHistoricalProjection::QueryRows => {
                return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
                    schema_version: query.schema_version().clone(),
                });
            }
        });
        let declared_history = physical
            .total_history_bytes
            .max(physical.inline_history_bytes);
        let preflight_bytes: u64 = (base_bytes as u64).saturating_add(match projection {
            AdmittedHistoricalProjection::Full => declared_history,
            AdmittedHistoricalProjection::KeysOnly
            | AdmittedHistoricalProjection::HeadOnly
            | AdmittedHistoricalProjection::QueryRows => 0,
        });
        if preflight_bytes > window_u64 {
            if !collected.is_empty() {
                return Ok(true);
            }
            return Err(HistoricalReadDenial::MemoryExhausted {
                requested_bytes: preflight_bytes,
                allowance_bytes: window_u64,
            });
        }
        if !collected.is_empty()
            && (*accumulated_bytes as u64).saturating_add(preflight_bytes) > window_u64
        {
            return Ok(true);
        }

        let remaining_history_budget =
            streaming_window_bytes.saturating_sub(accumulated_bytes.saturating_add(base_bytes));
        let (payload, history_versions) = match projection {
            AdmittedHistoricalProjection::KeysOnly => (Vec::new(), Vec::new()),
            AdmittedHistoricalProjection::HeadOnly => (physical.payload.to_vec(), Vec::new()),
            AdmittedHistoricalProjection::Full => {
                let history = reconstruct_borrowed_row_history_verified(
                    descriptor,
                    physical,
                    Some((remaining_history_budget, window_u64)),
                    &mut read_continuation,
                )?;
                (physical.payload.to_vec(), history)
            }
            AdmittedHistoricalProjection::QueryRows => {
                return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
                    schema_version: query.schema_version().clone(),
                });
            }
        };
        let row_bytes = physical
            .logical_key
            .len()
            .saturating_add(payload.len())
            .saturating_add(
                history_versions
                    .iter()
                    .map(Vec::len)
                    .fold(0usize, usize::saturating_add),
            );
        if row_bytes > streaming_window_bytes {
            if !collected.is_empty() {
                return Ok(true);
            }
            return Err(HistoricalReadDenial::MemoryExhausted {
                requested_bytes: row_bytes as u64,
                allowance_bytes: window_u64,
            });
        }
        if !collected.is_empty()
            && accumulated_bytes.saturating_add(row_bytes) > streaming_window_bytes
        {
            return Ok(true);
        }
        let owned_key = physical.logical_key.to_vec();
        let key = CanonicalCursorKey::after(
            PartitionId(physical.partition_id),
            physical.domain,
            owned_key.clone(),
        )
        .ok_or_else(corrupt)?;
        *accumulated_bytes = accumulated_bytes.saturating_add(row_bytes);
        collected.push((
            key,
            HistoricalBasisRow {
                partition_id: PartitionId(physical.partition_id),
                domain: physical.domain,
                kind_id: KindId(physical.kind_id),
                logical_key: owned_key,
                payload,
                history_versions,
            },
        ));
    }
    Ok(false)
}

fn find_continuation_locator(
    descriptor: &SealedChunkDescriptor,
    cont_id: ChunkMemberId,
) -> Option<&[u8; 64]> {
    let idx = descriptor
        .continuation_members
        .iter()
        .position(|id| *id == cont_id)?;
    descriptor.continuation_locators.get(idx)
}

fn reconstruct_borrowed_row_history_verified<F>(
    descriptor: &SealedChunkDescriptor,
    physical: BorrowedChunkRow<'_>,
    window_limit: Option<(usize, u64)>,
    read_continuation: &mut F,
) -> Result<Vec<Vec<u8>>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId, Option<&[u8; 64]>) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>,
{
    let mut history_versions = Vec::new();
    let mut actual_bytes: u64 = 0;
    let mut hasher = Sha256::new();
    hasher.update(ROW_HISTORY_DOMAIN);

    visit_continuation_ids(
        physical.continuation_pages_raw,
        physical.continuation_page_count,
        |expected_seq, cont_id| {
            let locator = if descriptor.continuation_manifest_member.is_some() {
                None
            } else {
                if !descriptor.continuation_members.contains(&cont_id) {
                    return Err(corrupt());
                }
                find_continuation_locator(descriptor, cont_id)
            };
            let page_bytes = read_continuation(cont_id, locator)?;
            verify_member_digest(cont_id, &page_bytes)?;
            let page = decode_borrowed_continuation_page(&page_bytes)?;
            if page.partition_id != physical.partition_id
                || page.domain != physical.domain
                || page.logical_key != physical.logical_key
                || page.sequence != expected_seq as u32
            {
                return Err(corrupt());
            }
            visit_length_prefixed_slices(page.versions_raw, page.version_count, |version| {
                actual_bytes = actual_bytes.saturating_add(version.len() as u64);
                if let Some((remaining_cap, allowance_bytes)) = window_limit
                    && actual_bytes > remaining_cap as u64
                {
                    return Err(HistoricalReadDenial::MemoryExhausted {
                        requested_bytes: actual_bytes,
                        allowance_bytes,
                    });
                }
                hasher.update((version.len() as u32).to_le_bytes());
                hasher.update(version);
                history_versions.push(version.to_vec());
                Ok(())
            })?;
            drop(page_bytes);
            Ok(())
        },
    )?;

    visit_length_prefixed_slices(
        physical.inline_history_raw,
        physical.inline_history_count,
        |version| {
            actual_bytes = actual_bytes.saturating_add(version.len() as u64);
            if let Some((remaining_cap, allowance_bytes)) = window_limit
                && actual_bytes > remaining_cap as u64
            {
                return Err(HistoricalReadDenial::MemoryExhausted {
                    requested_bytes: actual_bytes,
                    allowance_bytes,
                });
            }
            hasher.update((version.len() as u32).to_le_bytes());
            hasher.update(version);
            history_versions.push(version.to_vec());
            Ok(())
        },
    )?;

    if actual_bytes != physical.total_history_bytes {
        return Err(corrupt());
    }
    let derived_history: [u8; 32] = hasher.finalize().into();
    if derived_history != physical.history_versions_sha256 {
        return Err(corrupt());
    }
    Ok(history_versions)
}

/// Reconstructs and commitment-verifies all canonical rows of one chunk.
pub fn reconstruct_verified_chunk_rows<F>(
    descriptor: &SealedChunkDescriptor,
    chunk_bytes: &[u8],
    mut read_continuation: F,
) -> Result<Vec<HistoricalBasisRow>, HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId) -> Result<Vec<u8>, HistoricalReadDenial>,
{
    verify_member_digest(descriptor.chunk_member, chunk_bytes)?;
    let mut ordered = decode_borrowed_chunk_rows(chunk_bytes)?;
    ordered.sort_by(|a, b| {
        (a.partition_id, a.domain, a.logical_key).cmp(&(b.partition_id, b.domain, b.logical_key))
    });
    if compute_chunk_semantic_commitment_from_borrowed(&ordered) != descriptor.semantic_commitment {
        return Err(corrupt());
    }
    let mut rows = Vec::with_capacity(ordered.len());
    for physical in ordered {
        let history_versions =
            reconstruct_borrowed_row_history_verified(descriptor, physical, None, &mut |id, _| {
                read_continuation(id).map(Arc::new)
            })?;
        rows.push(HistoricalBasisRow {
            partition_id: PartitionId(physical.partition_id),
            domain: physical.domain,
            kind_id: KindId(physical.kind_id),
            logical_key: physical.logical_key.to_vec(),
            payload: physical.payload.to_vec(),
            history_versions,
        });
    }
    Ok(rows)
}

/// Flattens a multi-layer physical chunk image into a brand-new single-layer
/// physical member while proving bit-identical `semantic_commitment`.
pub fn consolidate_chunk<F>(
    descriptor: &SealedChunkDescriptor,
    chunk_bytes: &[u8],
    mut read_continuation: F,
) -> Result<(SealedChunkDescriptor, Vec<u8>), HistoricalReadDenial>
where
    F: FnMut(ChunkMemberId) -> Result<Vec<u8>, HistoricalReadDenial>,
{
    verify_member_digest(descriptor.chunk_member, chunk_bytes)?;
    let borrowed = decode_borrowed_chunk_rows(chunk_bytes)?;
    let mut sorted_borrowed = borrowed;
    sorted_borrowed.sort_by(|a, b| {
        (a.partition_id, a.domain, a.logical_key).cmp(&(b.partition_id, b.domain, b.logical_key))
    });
    if compute_chunk_semantic_commitment_from_borrowed(&sorted_borrowed)
        != descriptor.semantic_commitment
    {
        return Err(corrupt());
    }
    for row in sorted_borrowed {
        let _ = reconstruct_borrowed_row_history_verified(descriptor, row, None, &mut |id, _| {
            read_continuation(id).map(Arc::new)
        })?;
    }
    let image = decode_chunk_image(chunk_bytes)?;
    let mut flat_rows: Vec<PhysicalChunkRow> = image.layers.into_iter().flatten().collect();
    flat_rows.sort_by(|a, b| {
        (a.partition_id, a.domain, &a.logical_key).cmp(&(b.partition_id, b.domain, &b.logical_key))
    });
    let refs: Vec<&PhysicalChunkRow> = flat_rows.iter().collect();
    if compute_chunk_semantic_commitment_from_physical(&refs) != descriptor.semantic_commitment {
        return Err(corrupt());
    }
    let flat_bytes = encode_chunk_image(&PhysicalChunkImage {
        layers: vec![flat_rows],
    });
    let flat_id = member_id(&flat_bytes);
    let updated = SealedChunkDescriptor {
        semantic_commitment: descriptor.semantic_commitment,
        min_key: descriptor.min_key.clone(),
        max_key: descriptor.max_key.clone(),
        chunk_member: flat_id,
        continuation_members: descriptor.continuation_members.clone(),
        chunk_locator: None,
        continuation_locators: Vec::new(),
        continuation_manifest_member: None,
        continuation_manifest_locator: None,
        continuation_count: descriptor.continuation_count,
    };
    Ok((updated, flat_bytes))
}
