use super::{
    durable_frame::{read_u16, read_u32, read_u64},
    physical_fields::{record_key, scope},
    tree_frame::read_tree_frame,
    tree_reference::{branches, ordered},
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use std::collections::BTreeSet;
use worth_foundational::PhysicalArtifactFamily;
use worth_store_physical_format::integrity_declarations::families::root::ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION;

pub(crate) fn read_root_routing(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_tree_frame(
        bytes,
        expected,
        8,
        ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
        counters,
    )?;
    if frame.level != 0 {
        return branches(frame.body, frame.width, expected);
    }
    let ChildScope::Tree { first, last, .. } = &expected.scope else {
        unreachable!()
    };
    scope(
        &frame.body[..24] == first.as_slice()
            && &frame.body[frame.body.len() - 88..frame.body.len() - 64] == last.as_slice(),
        88,
        frame.body.len(),
        Field::ManifestPointer,
    )?;
    let mut children = Vec::new();
    let mut placements = BTreeSet::new();
    let mut previous = None;
    for (index, entry) in frame.body.chunks_exact(88).enumerate() {
        let start = 88 + index * 88;
        scope(
            record_key(entry).is_some()
                && route_class_valid(entry, bytes[9])
                && entry[86..88] == [0; 2],
            start,
            88,
            Field::ManifestPointer,
        )?;
        if let Some(previous) = previous {
            scope(
                ordered(expected.family, previous, entry, true),
                start,
                24,
                Field::ManifestPointer,
            )?;
        }
        previous = Some(entry);
        match entry[24] {
            1 => {
                scope(
                    [
                        read_u64(entry, 32),
                        read_u64(entry, 40),
                        read_u64(entry, 48),
                        read_u64(entry, 56),
                        read_u64(entry, 64),
                    ]
                    .iter()
                    .all(|v| *v != 0)
                        && read_u32(entry, 80) != 0
                        && read_u16(entry, 84) != 0,
                    start + 32,
                    54,
                    Field::ManifestPointer,
                )?;
                scope(
                    placements.insert((
                        1,
                        read_u64(entry, 32),
                        read_u64(entry, 40),
                        u64::from(read_u16(entry, 84)),
                    )),
                    start + 32,
                    54,
                    Field::ManifestPointer,
                )?;
            }
            2 => {
                let extent = read_u64(entry, 40);
                let generation = read_u64(entry, 48);
                let arena = read_u64(entry, 32);
                let offset = read_u64(entry, 56);
                let allocated_bytes = read_u64(entry, 64);
                let logical_bytes = read_u64(entry, 72);
                scope(
                    extent != 0
                        && generation != 0
                        && logical_bytes != 0
                        && arena != 0
                        && allocated_bytes >= 104
                        && offset.checked_add(allocated_bytes).is_some()
                        && entry[80..86] == [0; 6],
                    start + 32,
                    54,
                    Field::ManifestPointer,
                )?;
                scope(
                    placements.insert((2, extent, 0, 0)),
                    start + 40,
                    8,
                    Field::ManifestPointer,
                )?;
                children.push(ChildExpectation {
                    path: format!("families/records/arenas/arena-{arena:016x}.data"),
                    family: PhysicalArtifactFamily::ExtentManifest,
                    generation,
                    format: expected.format,
                    offset,
                    length: Some(104),
                    checksum: None,
                    scope: ChildScope::ExtentManifest {
                        arena,
                        extent,
                        record: entry[..24].try_into().unwrap(),
                        logical_bytes,
                        allocated_bytes,
                    },
                });
            }
            _ => scope(false, start + 24, 1, Field::ManifestPointer)?,
        }
    }
    Ok(children)
}

fn route_class_valid(entry: &[u8], schema: u8) -> bool {
    let field = &entry[25..32];
    if schema == 2 {
        return field == [0; 7];
    }
    if schema != 3 || field[5..] != [0; 2] || field[4] > 2 {
        return false;
    }
    match field[0] {
        0 => field[..4] == [0; 4] && field[4] == 0,
        1 | 4 => field[1..4] == [0; 3],
        2 => (1..=14).contains(&field[1]) && field[2..4] == [0; 2],
        3 => field[1] == 0 && read_u16(field, 2) != 0,
        _ => false,
    }
}
