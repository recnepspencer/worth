use super::durable_frame::{read_durable_frame, read_u16, read_u32, read_u64};
use super::physical_fields::{format_scope, scope, shape};
use crate::integrity_observation::child_expectation::{ChildExpectation, ChildScope};
use crate::integrity_observation::{
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use worth_foundational::PhysicalArtifactFamily;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityFormatDeclaration;

pub(crate) struct TreeFrame<'a> {
    pub(crate) body: &'a [u8],
    pub(crate) level: u16,
    pub(crate) count: u16,
    pub(crate) width: usize,
}

pub(crate) fn read_tree_frame<'a>(
    bytes: &'a [u8],
    expected: &ChildExpectation,
    kind: u8,
    declaration: PhysicalIntegrityFormatDeclaration,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<TreeFrame<'a>, OfflineIntegrityOutcome> {
    let ChildScope::Tree {
        tree,
        block,
        level,
        capacity,
        ..
    } = &expected.scope
    else {
        unreachable!("tree reader requires tree scope")
    };
    let width = match (expected.family, *level) {
        (PhysicalArtifactFamily::RootRoutingBlock, 0) => 88,
        (PhysicalArtifactFamily::RootRoutingBlock, _) => 72,
        (_, 0) => 40,
        _ => 56,
    };
    // A bounded count and the independently addressed level constrain a short
    // frame's missing suffix. This is diagnostic arithmetic, never allocation or
    // parent authority. Complete envelopes still reach checksum validation before
    // interpreting their count; corrupt count bytes must not hide a bad checksum.
    let expected_bytes = if bytes.len() >= 48 {
        let payload = u64::from(read_u32(bytes, 24));
        let records = payload.saturating_sub(40);
        let count = records / width as u64;
        let canonical = payload + 48;
        if payload > 40
            && records % width as u64 == 0
            && count <= u64::from(*capacity)
            && (bytes.len() < 68 || u64::from(read_u16(bytes, 66)) == count)
            && (bytes.len() as u64) < canonical
        {
            canonical as usize // bounded u16 capacity times a fixed record width
        } else {
            bytes.len()
        }
    } else {
        bytes.len().max(88)
    };
    let frame = read_durable_frame(bytes, expected_bytes, kind, declaration, counters)?;
    format_scope(&frame, expected.format)?;
    shape(frame.payload.len() >= 40, 24, 4)?;
    scope(frame.identity == *block, 28, 8, Field::FrameIdentity)?;
    scope(
        read_u64(frame.payload, 0) == *tree,
        48,
        8,
        Field::TreeIdentity,
    )?;
    scope(
        read_u64(frame.payload, 8) == *block,
        56,
        8,
        Field::IdentityField,
    )?;
    scope(
        read_u64(frame.payload, 24) == expected.generation,
        72,
        8,
        Field::ManifestGeneration,
    )?;
    scope(
        read_u16(frame.payload, 16) == *level,
        64,
        2,
        Field::TreeLevel,
    )?;
    shape(
        frame.payload[21..24] == [0; 3] && frame.payload[32..40] == [0; 8],
        69,
        19,
    )?;
    let count = read_u16(frame.payload, 18);
    shape(count > 0 && count <= *capacity, 66, 2)?;
    shape(frame.payload[20] == if *level == 0 { 1 } else { 2 }, 68, 1)?;
    shape(
        frame.payload.len() == 40 + usize::from(count) * width,
        24,
        4,
    )?;
    Ok(TreeFrame {
        body: &frame.payload[40..],
        level: *level,
        count,
        width,
    })
}
