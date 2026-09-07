use super::durable_frame::{read_u16, read_u32, read_u64};
use super::physical_fields::scope;
use crate::integrity_observation::child_expectation::{tree_path, ChildExpectation, ChildScope};
use crate::integrity_observation::{OfflineIntegrityOutcome, OfflinePhysicalFormatField as Field};
use worth_foundational::PhysicalArtifactFamily;

pub(crate) fn reference(
    bytes: &[u8],
    family: PhysicalArtifactFamily,
    tree: u64,
    capacity: u16,
    format: [u8; 10],
) -> ChildExpectation {
    let generation = read_u64(bytes, 0);
    let block = read_u64(bytes, 8);
    let width = (bytes.len() - 24) / 2;
    ChildExpectation {
        path: tree_path(family, generation, block),
        family,
        generation,
        format,
        offset: 0,
        length: None,
        checksum: Some(read_u32(bytes, 20)),
        scope: ChildScope::Tree {
            tree,
            block,
            level: read_u16(bytes, 16),
            capacity,
            first: bytes[24..24 + width].to_vec(),
            last: bytes[24 + width..].to_vec(),
        },
    }
}

pub(crate) fn branches(
    body: &[u8],
    width: usize,
    expected: &ChildExpectation,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let ChildScope::Tree {
        tree,
        capacity,
        level,
        first,
        last,
        ..
    } = &expected.scope
    else {
        unreachable!()
    };
    let mut children = Vec::with_capacity(body.len() / width);
    let mut previous = None;
    for (index, bytes) in body.chunks_exact(width).enumerate() {
        let start = 88 + index * width;
        let child = reference(bytes, expected.family, *tree, *capacity, expected.format);
        let ChildScope::Tree {
            block,
            level: child_level,
            first: child_first,
            last: child_last,
            ..
        } = &child.scope
        else {
            unreachable!()
        };
        scope(
            child.generation != 0
                && child.generation <= expected.generation
                && *block != 0
                && child_level.checked_add(1) == Some(*level),
            start,
            20,
            Field::ManifestPointer,
        )?;
        scope(
            bytes[18..20] == [0; 2] && ordered(expected.family, child_first, child_last, false),
            start + 18,
            width - 18,
            Field::ManifestPointer,
        )?;
        if let Some(prior) = previous {
            scope(
                ordered(expected.family, prior, child_first, true),
                start + 24,
                width - 24,
                Field::ManifestPointer,
            )?;
        }
        previous = Some(&bytes[24 + (width - 24) / 2..]);
        children.push(child);
    }
    let ChildScope::Tree {
        first: actual_first,
        ..
    } = &children.first().expect("nonempty tree").scope
    else {
        unreachable!()
    };
    let ChildScope::Tree {
        last: actual_last, ..
    } = &children.last().expect("nonempty tree").scope
    else {
        unreachable!()
    };
    scope(
        actual_first == first && actual_last == last,
        88,
        body.len(),
        Field::ManifestPointer,
    )?;
    Ok(children)
}

pub(crate) fn ordered(
    family: PhysicalArtifactFamily,
    left: &[u8],
    right: &[u8],
    strict: bool,
) -> bool {
    let ordering = match family {
        PhysicalArtifactFamily::RootRoutingBlock => match (
            super::physical_fields::record_key(left),
            super::physical_fields::record_key(right),
        ) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => return false,
        },
        PhysicalArtifactFamily::SegmentMembershipBlock => {
            if [
                read_u64(left, 0),
                read_u64(left, 8),
                read_u64(right, 0),
                read_u64(right, 8),
            ]
            .contains(&0)
            {
                return false;
            }
            (read_u64(left, 0), read_u64(left, 8)).cmp(&(read_u64(right, 0), read_u64(right, 8)))
        }
        PhysicalArtifactFamily::FreeSpaceMembershipBlock => {
            if !matches!(left[0], 1 | 2)
                || !matches!(right[0], 1 | 2)
                || left[1..8] != [0; 7]
                || right[1..8] != [0; 7]
                || read_u64(left, 8) == 0
                || read_u64(right, 8) == 0
            {
                return false;
            }
            (left[0], read_u64(left, 8)).cmp(&(right[0], read_u64(right, 8)))
        }
        _ => return false,
    };
    if strict {
        ordering.is_lt()
    } else {
        !ordering.is_gt()
    }
}
