use std::collections::BTreeMap;
use std::path::Path;

use worth_foundational::PhysicalArtifactFamily;

use super::super::blob_record::{self, BlobFact};
use super::super::child_expectation::{ChildExpectation, ChildScope};
use super::super::families::extent::read_extent_chunk;
use super::super::sha256::Sha256;
use super::super::{
    BoundedMediaWalk, OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason,
};
use super::{damage, graph::dependency_uncertainty, Selected};

/// The first pass proves selected C.5 routes and all inner claims. This second
/// pass follows verified tree order, reads one routed chunk at a time through
/// the same bounded acquisition owner, and recomputes portable plaintext SHA.
pub(super) fn verify_publications(
    selected: &mut [Selected],
    root: &Path,
    walk: &mut BoundedMediaWalk,
) {
    let records: BTreeMap<[u8; 24], usize> = selected
        .iter()
        .enumerate()
        .filter(|(_, row)| row.fact.is_some())
        .map(|(index, row)| (row.record, index))
        .collect();
    for index in 0..selected.len() {
        let Some(BlobFact::Publication {
            root: tree_root,
            total,
            logical_digest,
            ..
        }) = selected[index].fact.as_ref()
        else {
            continue;
        };
        if selected[index].outcome != Outcome::Intact {
            continue;
        }
        let mut hasher = Sha256::new();
        let mut bytes = 0_u64;
        let result = visit_node(*tree_root, selected, &records, 0, &mut |child| {
            let Some(BlobFact::Chunk { length, .. }) = &child.fact else {
                return Err(damage(Cause::Pointer));
            };
            let payload = reread_selected_chunk(child, root, walk)?;
            let frame =
                blob_record::decode(&payload, Some(child_store(child)?), walk.counters_mut())?;
            if Some(&frame) != child.fact.as_ref() || payload.len() < 140 {
                return Err(damage(Cause::ChecksumMismatch));
            }
            let plaintext = &payload[140..];
            if plaintext.len() as u64 != *length {
                return Err(damage(Cause::ScopeMismatch));
            }
            hasher.update(plaintext);
            bytes = bytes
                .checked_add(*length)
                .ok_or_else(|| damage(Cause::Framing))?;
            Ok(())
        });
        let outcome = match result {
            Err(outcome) => outcome,
            Ok(()) if bytes == *total => {
                walk.counters_mut().checksum_calculations += 1;
                if hasher.finish() == *logical_digest {
                    Outcome::Intact
                } else {
                    damage(Cause::ChecksumMismatch)
                }
            }
            Ok(()) => damage(Cause::ScopeMismatch),
        };
        selected[index].outcome = outcome;
    }
}

fn visit_node<F: FnMut(&Selected) -> Result<(), Outcome>>(
    record: [u8; 24],
    selected: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    depth: u16,
    on_chunk: &mut F,
) -> Result<(), Outcome> {
    if depth > 255 {
        return Err(damage(Cause::Pointer));
    }
    let row = selected
        .get(*records.get(&record).ok_or_else(|| damage(Cause::Pointer))?)
        .ok_or_else(|| damage(Cause::Pointer))?;
    if row.outcome != Outcome::Intact {
        return Err(dependency_uncertainty(row).unwrap_or_else(|| damage(Cause::Pointer)));
    }
    let Some(BlobFact::Node { kind, entries, .. }) = &row.fact else {
        return Err(damage(Cause::Pointer));
    };
    for edge in entries {
        if *kind == 2 {
            visit_node(edge.record, selected, records, depth + 1, on_chunk)?;
        } else {
            let child = selected
                .get(
                    *records
                        .get(&edge.record)
                        .ok_or_else(|| damage(Cause::Pointer))?,
                )
                .ok_or_else(|| damage(Cause::Pointer))?;
            if child.outcome != Outcome::Intact {
                return Err(dependency_uncertainty(child).unwrap_or_else(|| damage(Cause::Pointer)));
            }
            let Some(BlobFact::Chunk { .. }) = &child.fact else {
                return Err(damage(Cause::Pointer));
            };
            on_chunk(child)?;
        }
    }
    Ok(())
}

fn child_store(row: &Selected) -> Result<[u8; 16], Outcome> {
    row.fact
        .as_ref()
        .map(BlobFact::store)
        .ok_or_else(|| damage(Cause::Pointer))
}

pub(super) fn reread_selected_chunk(
    row: &Selected,
    root: &Path,
    walk: &mut BoundedMediaWalk,
) -> Result<Vec<u8>, Outcome> {
    let route = row.route.as_ref().ok_or_else(|| damage(Cause::Pointer))?;
    let Some((first, _)) = route.frames.first().copied() else {
        return Err(damage(Cause::Pointer));
    };
    let Some((last_offset, last_length)) = route.frames.last().copied() else {
        return Err(damage(Cause::Pointer));
    };
    let end = last_offset
        .checked_add(last_length)
        .ok_or_else(|| damage(Cause::Framing))?;
    let length = end
        .checked_sub(first)
        .ok_or_else(|| damage(Cause::Framing))?;
    let acquired = walk.acquire_range(&root.join(&row.path), 4, first, length)?;
    if acquired.physical_alias_of.is_some() {
        return Err(Outcome::Unknown(
            OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected,
        ));
    }
    if acquired.bytes.len() as u64 != length {
        return Err(super::super::record_walk::damage(
            Cause::Truncation,
            Some((first, length)),
            Blast::Artifact,
        ));
    }
    let mut payload = Vec::with_capacity(route.logical_bytes as usize);
    for (index, (offset, frame_length)) in route.frames.iter().copied().enumerate() {
        let start = usize::try_from(offset - first).map_err(|_| damage(Cause::Framing))?;
        let end = start
            .checked_add(usize::try_from(frame_length).map_err(|_| damage(Cause::Framing))?)
            .ok_or_else(|| damage(Cause::Framing))?;
        let frame = acquired
            .bytes
            .get(start..end)
            .ok_or_else(|| damage(Cause::Truncation))?;
        let expected = ChildExpectation {
            path: row.path.clone(),
            family: PhysicalArtifactFamily::ExtentChunkFrame,
            generation: row.generation,
            format: route.format,
            offset,
            length: Some(frame_length),
            checksum: None,
            scope: ChildScope::ExtentChunk {
                arena: route.arena,
                extent: route.extent,
                record: row.record,
                logical_bytes: route.logical_bytes,
                logical_offset: payload.len() as u64,
                ordinal: (index + 1) as u32,
            },
        };
        read_extent_chunk(frame, &expected, walk.counters_mut())?;
        payload.extend_from_slice(&frame[112..]);
    }
    if payload.len() as u64 != route.logical_bytes {
        return Err(damage(Cause::Framing));
    }
    Ok(payload)
}

#[cfg(test)]
mod tests;
