use super::{
    durable_frame::{read_durable_frame, read_u32, read_u64},
    physical_fields::{format_scope, scope, shape},
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use worth_foundational::PhysicalArtifactFamily;
use worth_store_physical_format::integrity_declarations::families::{
    EXTENT_CHUNK_INTEGRITY_DECLARATION, EXTENT_MANIFEST_INTEGRITY_DECLARATION,
};

pub(crate) fn read_extent_manifest(
    bytes: &[u8],
    expected: &ChildExpectation,
    maximum_children: u64,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        104,
        6,
        EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        counters,
    )?;
    format_scope(&frame, expected.format)?;
    let ChildScope::ExtentManifest {
        extent,
        record,
        logical_bytes,
    } = expected.scope
    else {
        unreachable!()
    };
    scope(
        frame.identity == expected.generation,
        28,
        8,
        Field::FrameIdentity,
    )?;
    scope(
        frame.payload[..24] == record && read_u64(frame.payload, 24) == extent,
        48,
        32,
        Field::IdentityField,
    )?;
    scope(
        read_u64(frame.payload, 32) == logical_bytes,
        80,
        8,
        Field::PayloadLength,
    )?;
    let page_bytes = read_u32(&expected.format, 2);
    let payload_bytes = u64::from(page_bytes - 112);
    let chunks = logical_bytes.div_ceil(payload_bytes);
    shape(
        read_u32(frame.payload, 40) == page_bytes
            && u64::from(read_u32(frame.payload, 44)) == chunks
            && frame.payload[48..56] == [0; 8],
        88,
        16,
    )?;
    if chunks > maximum_children {
        return Err(OfflineIntegrityOutcome::Indeterminate(
            crate::OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
        ));
    }
    // Cardinality is charged before the caller retains these expectations.
    let mut children = Vec::new();
    for index in 0..chunks {
        let logical_offset = index * payload_bytes;
        let length = 112 + (logical_bytes - logical_offset).min(payload_bytes);
        children.push(ChildExpectation {
            path: format!(
                "families/records/extents/extent-{extent:016x}-{:016x}.data",
                expected.generation
            ),
            family: PhysicalArtifactFamily::ExtentChunkFrame,
            generation: expected.generation,
            format: expected.format,
            offset: index * u64::from(page_bytes),
            length: Some(length),
            checksum: None,
            scope: ChildScope::ExtentChunk {
                extent,
                record,
                logical_bytes,
                logical_offset,
                ordinal: (index + 1) as u32,
            },
        });
    }
    Ok(children)
}

pub(crate) fn read_extent_chunk(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        expected.length.unwrap() as usize,
        4,
        EXTENT_CHUNK_INTEGRITY_DECLARATION,
        counters,
    )?;
    format_scope(&frame, expected.format)?;
    let ChildScope::ExtentChunk {
        extent,
        record,
        logical_bytes,
        logical_offset,
        ordinal,
    } = expected.scope
    else {
        unreachable!()
    };
    shape(frame.payload.len() >= 64, 24, 4)?;
    scope(
        frame.identity == u64::from(ordinal),
        28,
        8,
        Field::FrameIdentity,
    )?;
    scope(
        frame.payload[..24] == record && read_u64(frame.payload, 24) == extent,
        48,
        32,
        Field::IdentityField,
    )?;
    scope(
        read_u64(frame.payload, 32) == expected.generation,
        80,
        8,
        Field::ManifestGeneration,
    )?;
    scope(
        read_u64(frame.payload, 40) == logical_bytes
            && read_u64(frame.payload, 48) == logical_offset,
        88,
        16,
        Field::IdentityField,
    )?;
    shape(
        read_u32(frame.payload, 56) as usize == frame.payload.len() - 64
            && frame.payload[60..64] == [0; 4],
        104,
        8,
    )?;
    Ok(Vec::new())
}
