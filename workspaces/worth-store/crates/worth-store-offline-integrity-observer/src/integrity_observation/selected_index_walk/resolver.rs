use std::path::Path;

use super::super::families::root_manifest::OfflineRootManifestFacts;
use super::super::{
    child_expectation::{ChildExpectation, ChildScope},
    families::{
        durable_frame::{read_u16, read_u32, read_u64},
        physical_fields::record_key,
        tree_reference::reference,
    },
    record_walk::{damage, inspect_expected},
    BoundedMediaWalk, OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason,
};
use worth_foundational::PhysicalArtifactFamily as Family;

pub(super) type RecordKey = ([u8; 16], u64);

pub(super) struct ResolvedRecord {
    pub(super) path: String,
    pub(super) offset: Option<u64>,
    pub(super) payload: Vec<u8>,
}

pub(super) fn resolve(
    root: &Path,
    manifest: &OfflineRootManifestFacts,
    record: RecordKey,
    walk: &mut BoundedMediaWalk,
) -> Result<ResolvedRecord, Outcome> {
    let payload = &manifest.payload;
    if payload[40] != 1 {
        return Err(pointer());
    }
    let tree = read_u64(payload, 8);
    let capacity = read_u16(payload, 16);
    let routing = reference(
        &payload[48..120],
        Family::RootRoutingBlock,
        tree,
        capacity,
        manifest.format,
    );
    let (_, routing_bytes, _) = descend(root, routing, RouteKey::Record(record), walk)?;
    let route = routing_bytes[88..]
        .chunks_exact(88)
        .find(|entry| record_key(entry) == Some(record))
        .ok_or_else(pointer)?;
    match route[24] {
        1 => resolve_inline(root, manifest, record, route, walk),
        2 => resolve_extent(root, manifest, record, route, walk),
        _ => Err(pointer()),
    }
}

fn resolve_inline(
    root: &Path,
    manifest: &OfflineRootManifestFacts,
    record: RecordKey,
    route: &[u8],
    walk: &mut BoundedMediaWalk,
) -> Result<ResolvedRecord, Outcome> {
    let payload = &manifest.payload;
    let tree = read_u64(payload, 8);
    let capacity = read_u16(payload, 16);
    let segment = read_u64(route, 32);
    let page = read_u64(route, 40);
    let segment_generation = read_u64(route, 48);
    let page_generation = read_u64(route, 56);
    let slot_generation = read_u64(route, 64);
    let payload_length = read_u64(route, 72);
    let pages = read_u32(route, 80);
    let slot = read_u16(route, 84);
    if slot == 0 || payload_length == 0 || pages == 0 {
        return Err(pointer());
    }
    if payload[160] != 1 {
        return Err(pointer());
    }
    let membership = reference(
        &payload[168..224],
        Family::SegmentMembershipBlock,
        tree,
        capacity,
        manifest.format,
    );
    let (_, _, children) = descend(root, membership, RouteKey::Page(segment, page), walk)?;
    // Route capacity, global page identity, and selected physical frame count
    // are different dimensions; membership itself validates frame bounds.
    let expected = children.into_iter().find(|child| matches!(child.scope, ChildScope::Page { segment: found_segment, page: found_page, .. } if found_segment == segment && found_page == page))
        .ok_or_else(pointer)?;
    if expected.generation != page_generation
        || !expected
            .path
            .ends_with(&format!("-{segment_generation:016x}.pages"))
    {
        return Err(pointer());
    }
    let page_bytes = acquire_expected(root, &expected, walk)?;
    inspect_expected(&page_bytes, &expected, walk)?;
    let slot_base = 72usize
        .checked_add(usize::from(slot - 1).checked_mul(40).ok_or_else(pointer)?)
        .ok_or_else(pointer)?;
    let slot_bytes = page_bytes
        .get(slot_base..slot_base + 40)
        .ok_or_else(pointer)?;
    let count = read_u16(&page_bytes, 64);
    if slot > count
        || record_key(slot_bytes) != Some(record)
        || read_u64(slot_bytes, 32) != slot_generation
    {
        return Err(pointer());
    }
    let start = 48usize
        .checked_add(read_u32(slot_bytes, 24) as usize)
        .ok_or_else(pointer)?;
    let length = read_u32(slot_bytes, 28) as usize;
    if length as u64 != payload_length {
        return Err(pointer());
    }
    let end = start.checked_add(length).ok_or_else(pointer)?;
    let bytes = page_bytes.get(start..end).ok_or_else(pointer)?;
    Ok(ResolvedRecord {
        path: expected.path,
        offset: Some(expected.offset + start as u64),
        payload: bytes.to_vec(),
    })
}

fn resolve_extent(
    root: &Path,
    manifest: &OfflineRootManifestFacts,
    record: RecordKey,
    route: &[u8],
    walk: &mut BoundedMediaWalk,
) -> Result<ResolvedRecord, Outcome> {
    let arena = read_u64(route, 32);
    let extent = read_u64(route, 40);
    let generation = read_u64(route, 48);
    let offset = read_u64(route, 56);
    let allocated_bytes = read_u64(route, 64);
    let logical_bytes = read_u64(route, 72);
    if logical_bytes == 0 || logical_bytes > 65_424 {
        return Err(damage(Cause::Framing, None, Blast::Artifact));
    }
    if logical_bytes
        > walk
            .maximum_entries()
            .saturating_mul(u64::from(read_u32(&manifest.format, 2)))
    {
        return Err(walk.entry_bound());
    }
    let mut record_bytes = [0; 24];
    record_bytes[..16].copy_from_slice(&record.0);
    record_bytes[16..24].copy_from_slice(&record.1.to_le_bytes());
    let expected = ChildExpectation {
        path: format!("families/records/arenas/arena-{arena:016x}.data"),
        family: Family::ExtentManifest,
        generation,
        format: manifest.format,
        offset,
        length: Some(104),
        checksum: None,
        scope: ChildScope::ExtentManifest {
            arena,
            extent,
            record: record_bytes,
            logical_bytes,
            allocated_bytes,
        },
    };
    let manifest_bytes = acquire_expected(root, &expected, walk)?;
    let chunks = inspect_expected(&manifest_bytes, &expected, walk)?;
    let mut payload = Vec::with_capacity(logical_bytes as usize);
    for chunk in chunks {
        let bytes = acquire_expected(root, &chunk, walk)?;
        inspect_expected(&bytes, &chunk, walk)?;
        payload.extend_from_slice(bytes.get(112..).ok_or_else(pointer)?);
    }
    if payload.len() as u64 != logical_bytes {
        return Err(pointer());
    }
    Ok(ResolvedRecord {
        path: expected.path,
        offset: None,
        payload,
    })
}

enum RouteKey {
    Record(RecordKey),
    Page(u64, u64),
}

fn descend(
    root: &Path,
    mut expected: ChildExpectation,
    key: RouteKey,
    walk: &mut BoundedMediaWalk,
) -> Result<(ChildExpectation, Vec<u8>, Vec<ChildExpectation>), Outcome> {
    let mut depth = 0_u64;
    loop {
        if depth >= walk.maximum_entries() {
            return Err(walk.entry_bound());
        }
        depth += 1;
        let bytes = acquire_expected(root, &expected, walk)?;
        let children = inspect_expected(&bytes, &expected, walk)?;
        if matches!(expected.scope, ChildScope::Tree { level: 0, .. }) {
            return Ok((expected, bytes, children));
        }
        expected = children
            .into_iter()
            .find(|child| contains(child, &key))
            .ok_or_else(pointer)?;
    }
}

fn contains(child: &ChildExpectation, key: &RouteKey) -> bool {
    let ChildScope::Tree { first, last, .. } = &child.scope else {
        return false;
    };
    match key {
        RouteKey::Record(key) => record_key(first)
            .zip(record_key(last))
            .is_some_and(|(low, high)| low <= *key && *key <= high),
        RouteKey::Page(segment, page) => {
            let tuple = |bytes: &[u8]| (read_u64(bytes, 0), read_u64(bytes, 8));
            tuple(first) <= (*segment, *page) && (*segment, *page) <= tuple(last)
        }
    }
}

fn acquire_expected(
    root: &Path,
    expected: &ChildExpectation,
    walk: &mut BoundedMediaWalk,
) -> Result<Vec<u8>, Outcome> {
    let path = root.join(&expected.path);
    if !path.try_exists().unwrap_or(true) {
        walk.counters_mut().missing_artifacts += 1;
        return Err(damage(Cause::MissingArtifact, None, Blast::Artifact));
    }
    let extent_frame = matches!(
        expected.scope,
        ChildScope::ExtentManifest { .. } | ChildScope::ExtentChunk { .. }
    );
    let acquired = if extent_frame {
        walk.acquire_range(
            &path,
            4,
            expected.offset,
            expected.length.ok_or_else(pointer)?,
        )?
    } else {
        walk.acquire(&path, 4)?
    };
    if acquired.is_alias() {
        return Err(Outcome::Unknown(
            OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected,
        ));
    }
    if let ChildScope::Page { pages, .. } = expected.scope {
        let required = u64::from(pages) * u64::from(read_u32(&expected.format, 2));
        if acquired.byte_length as u64 != required {
            return Err(damage(
                Cause::Framing,
                Some((0, acquired.byte_length.max(1) as u64)),
                Blast::Artifact,
            ));
        }
    }
    let start = if extent_frame {
        0
    } else {
        usize::try_from(expected.offset).map_err(|_| pointer())?
    };
    let end = if extent_frame {
        expected
            .length
            .and_then(|length| usize::try_from(length).ok())
            .ok_or_else(pointer)?
    } else {
        expected
            .length
            .and_then(|length| expected.offset.checked_add(length))
            .and_then(|end| usize::try_from(end).ok())
            .unwrap_or(acquired.byte_length)
    };
    acquired
        .bytes
        .get(start..end)
        .map(|bytes| bytes.to_vec())
        .ok_or_else(|| {
            damage(
                Cause::Truncation,
                Some((start as u64, end.saturating_sub(start) as u64)),
                Blast::Artifact,
            )
        })
}

fn pointer() -> Outcome {
    damage(Cause::Pointer, None, Blast::ReachableRootSubtree)
}
