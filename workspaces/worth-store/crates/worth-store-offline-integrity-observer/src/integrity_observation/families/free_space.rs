use super::{
    durable_frame::{read_durable_frame, read_u16, read_u32, read_u64},
    physical_fields::{format_scope, scope, shape},
    tree_frame::read_tree_frame,
    tree_reference::{branches, ordered, reference},
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use worth_foundational::PhysicalArtifactFamily;
use worth_store_physical_format::integrity_declarations::families::free_space::{
    FREE_SPACE_HEADER_INTEGRITY_DECLARATION, FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
};

pub(crate) fn read_free_space_header(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        176,
        7,
        FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
        counters,
    )?;
    format_scope(&frame, expected.format)?;
    let ChildScope::FreeSpace { tree, capacity } = expected.scope else {
        unreachable!()
    };
    let payload = frame.payload;
    scope(
        frame.identity == expected.generation && read_u64(payload, 0) == expected.generation,
        28,
        28,
        Field::ManifestGeneration,
    )?;
    scope(read_u64(payload, 8) == tree, 56, 8, Field::TreeIdentity)?;
    scope(
        read_u16(payload, 16) >= 2 && read_u16(payload, 16) <= capacity,
        64,
        2,
        Field::IdentityField,
    )?;
    shape(
        payload[22..24] == [0; 2] && payload[65..72] == [0; 7] && payload[64] <= 1,
        70,
        50,
    )?;
    scope(
        read_u32(payload, 18) != 0
            && u64::from(read_u32(payload, 18))
                <= u64::from(read_u32(&expected.format, 2) - 72) / 40
            && [32, 40, 48, 56]
                .iter()
                .all(|offset| read_u64(payload, *offset) != 0),
        66,
        46,
        Field::ManifestPointer,
    )?;
    if payload[64] == 0 {
        scope(read_u64(payload, 24) == 0, 72, 8, Field::ManifestPointer)?;
        return Ok(Vec::new());
    }
    scope(read_u64(payload, 24) != 0, 72, 8, Field::ManifestPointer)?;
    let child = reference(
        &payload[72..128],
        PhysicalArtifactFamily::FreeSpaceMembershipBlock,
        tree,
        capacity,
        expected.format,
    );
    let ChildScope::Tree {
        block, first, last, ..
    } = &child.scope
    else {
        unreachable!()
    };
    scope(
        child.generation != 0
            && child.generation <= expected.generation
            && *block != 0
            && *block < read_u64(payload, 56)
            && ordered(child.family, first, last, false),
        120,
        56,
        Field::ManifestPointer,
    )?;
    Ok(vec![child])
}

pub(crate) fn read_free_space_membership(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_tree_frame(
        bytes,
        expected,
        10,
        FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
        counters,
    )?;
    if frame.level != 0 {
        return branches(frame.body, frame.width, expected);
    }
    let ChildScope::Tree { first, last, .. } = &expected.scope else {
        unreachable!()
    };
    scope(
        &frame.body[..16] == first.as_slice()
            && &frame.body[frame.body.len() - 40..frame.body.len() - 24] == last.as_slice(),
        88,
        frame.body.len(),
        Field::ManifestPointer,
    )?;
    let mut previous = None;
    for (index, entry) in frame.body.chunks_exact(40).enumerate() {
        let start = 88 + index * 40;
        scope(
            matches!(entry[0], 1 | 2)
                && entry[1..8] == [0; 7]
                && read_u64(entry, 8) != 0
                && read_u64(entry, 16) != 0
                && read_u64(entry, 32) != 0
                && read_u64(entry, 16)
                    .checked_add(read_u64(entry, 24))
                    .is_some(),
            start,
            40,
            Field::ManifestPointer,
        )?;
        if let Some(previous) = previous {
            scope(
                ordered(expected.family, previous, entry, true),
                start,
                16,
                Field::ManifestPointer,
            )?;
        }
        previous = Some(entry);
    }
    Ok(Vec::new())
}
