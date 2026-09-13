use super::{
    durable_frame::{read_u32, read_u64},
    physical_fields::scope,
    tree_frame::read_tree_frame,
    tree_reference::{branches, ordered},
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use worth_foundational::PhysicalArtifactFamily;
use worth_store_physical_format::integrity_declarations::families::SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION;

pub(crate) fn read_segment_membership(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_tree_frame(
        bytes,
        expected,
        9,
        SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
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
    let mut children = Vec::with_capacity(usize::from(frame.count));
    let mut previous = None;
    let page_bytes = u64::from(read_u32(&expected.format, 2));
    for (index, entry) in frame.body.chunks_exact(40).enumerate() {
        let start = 88 + index * 40;
        let segment = read_u64(entry, 0);
        let page = read_u64(entry, 8);
        let generation = read_u64(entry, 16);
        let data_generation = read_u64(entry, 24);
        let pages = read_u32(entry, 32);
        let frame_index = read_u32(entry, 36);
        scope(
            segment != 0
                && page != 0
                && generation != 0
                && data_generation != 0
                && pages != 0
                && frame_index < pages,
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
        children.push(ChildExpectation {
            path: format!(
                "families/records/segments/segment-{segment:016x}-{data_generation:016x}.pages"
            ),
            family: PhysicalArtifactFamily::PageFrame,
            generation,
            format: expected.format,
            offset: u64::from(frame_index) * page_bytes,
            length: Some(page_bytes),
            checksum: None,
            scope: ChildScope::Page {
                segment,
                page,
                pages,
            },
        });
    }
    Ok(children)
}
