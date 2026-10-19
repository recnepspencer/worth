//! Bounded wire for postcommit producer facts and native output expectations.
//! Unsupported decision facts are deliberately not checkpoint-reusable.

use std::sync::Arc;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, VersionId},
    runtime::{RelationalAdjacencyDirection, RelationalFieldPresence, RelationalFieldRevision},
};

use super::CheckpointCursor;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
mod fact_decode_denial;
pub(in crate::domain_computation::primary_graph) use fact_decode_denial::FactDecodeDenial;
mod encoding;
mod indexed_selection;
pub(in crate::domain_computation::primary_graph) use encoding::encode_funded;
use encoding::FactBytes;
mod payload;
mod record;
pub(in crate::domain_computation::primary_graph) use payload::{
    CodecPayloadReservation, FactPayload, FactSequence, NativeFactPatchBuilder,
    NativeFactPayloadBuilder,
};
mod stream;
pub(in crate::domain_computation::primary_graph) use stream::FactDecoder;

/// Producer facts promise no consumed outputs and no known own-write staleness.
/// Version 8 exports did not exclude own-write-stale computations; neither
/// their checkpoint format nor their fact payload is admitted as version 9.
/// Version 10 additionally carries exact native Field/AbsentField evidence;
/// these are not relabelled as query source revisions.
/// Version 11 distinguishes lawful Optional whole-aspect absence from a
/// missing field within a present aspect. Displaced version10 is unsupported.
pub(in crate::domain_computation::primary_graph) const WIRE_VERSION: u16 = 11;
pub(super) const MAXIMUM_FACT_BYTES: usize = 16 * 1024 * 1024;
// SourceEntity is the smallest current fact: one tag and a 16-byte EntityId.
const MINIMUM_FACT_WIRE_BYTES: usize = 17;
const MAXIMUM_SET_ENTITIES: usize = 4096;
const MAXIMUM_TEXT: usize = 4096;
const MAXIMUM_FIELD_DEPTH: usize = 32;

pub(in crate::domain_computation::primary_graph) fn encode(
    facts: &(impl FactSequence + ?Sized),
) -> Option<Vec<u8>> {
    encode_with_capacity(facts, 0)
}

pub(super) fn encode_with_capacity(
    facts: &(impl FactSequence + ?Sized),
    capacity: usize,
) -> Option<Vec<u8>> {
    if facts.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(capacity);
    write_facts(facts, &mut bytes)?;
    Some(bytes)
}
fn write_facts(facts: &(impl FactSequence + ?Sized), bytes: &mut impl FactBytes) -> Option<()> {
    if facts.is_empty() {
        return None;
    }
    put_u32(bytes, u32::try_from(facts.len()).ok()?);
    for fact in facts.iter() {
        match fact {
            Fact::RetiredOutputEntity {
                entity_id,
                kind,
                created_at,
                deleted_at,
                read_locator,
            } => {
                bytes.push(6);
                put_entity(bytes, *entity_id);
                put_u32(bytes, kind.as_u32());
                put_u64(bytes, created_at.as_u64());
                put_u64(bytes, deleted_at.as_u64());
                put_text(bytes, read_locator)?;
            }
            Fact::SourceEntity { entity_id } => {
                bytes.push(1);
                put_entity(bytes, *entity_id);
            }
            Fact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => {
                bytes.push(2);
                put_entity(bytes, *entity_id);
                put_text(bytes, aspect.as_str())?;
                put_optional_version(bytes, native_revision.map(VersionId));
            }
            Fact::SourceFieldRevision {
                entity_id,
                locator,
                native_revision,
            } => {
                let revision = native_revision.as_ref()?;
                bytes.push(3);
                put_entity(bytes, *entity_id);
                put_locator(bytes, locator)?;
                put_u64(bytes, revision.version().as_u64());
                bytes.push(match revision.presence() {
                    RelationalFieldPresence::Present => 1,
                    RelationalFieldPresence::Absent => 2,
                });
            }
            Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                endpoints,
            } => {
                if endpoints.len() > MAXIMUM_SET_ENTITIES {
                    return None;
                }
                bytes.push(4);
                put_u32(bytes, relation_kind.as_u32());
                put_entity(bytes, *anchor);
                bytes.push(match direction {
                    RelationalAdjacencyDirection::Outgoing => 1,
                    RelationalAdjacencyDirection::Incoming => 2,
                });
                put_optional_version(bytes, *native_revision);
                put_u64(bytes, u64::try_from(*comparison_work_limit).ok()?);
                put_u32(bytes, u32::try_from(endpoints.len()).ok()?);
                for endpoint in endpoints {
                    put_entity(bytes, *endpoint);
                }
            }
            Fact::Entity { entity_id, kind } => {
                bytes.push(5);
                put_entity(bytes, *entity_id);
                put_u32(bytes, kind.as_u32());
            }
            Fact::IndexedEntitySelection { .. } => {
                bytes.push(7);
                indexed_selection::encode(bytes, fact)?;
            }
            Fact::Field {
                entity_id,
                kind,
                locator,
                value,
            } => {
                bytes.push(8);
                put_entity(bytes, *entity_id);
                put_u32(bytes, kind.as_u32());
                put_locator(bytes, locator)?;
                let length = encoding::json_length(value, MAXIMUM_FACT_BYTES)?;
                put_u32(bytes, u32::try_from(length).ok()?);
                encoding::json(bytes, value, MAXIMUM_FACT_BYTES)?;
            }
            Fact::AbsentField {
                entity_id,
                kind,
                locator,
            } => {
                bytes.push(9);
                put_entity(bytes, *entity_id);
                put_u32(bytes, kind.as_u32());
                put_locator(bytes, locator)?;
            }
            Fact::OptionalAspectAbsent {
                entity_id,
                kind,
                locator,
            } => {
                bytes.push(10);
                put_entity(bytes, *entity_id);
                put_u32(bytes, kind.as_u32());
                put_locator(bytes, locator)?;
            }
            _ => return None,
        }
        if bytes.len() > MAXIMUM_FACT_BYTES {
            return None;
        }
    }
    Some(())
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn decode(
    bytes: &[u8],
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<FactPayload, fact_decode_denial::FactDecodeDenial> {
    decode_for_wire_version(bytes, WIRE_VERSION, request, allocation_policy)
}

pub(in crate::domain_computation::primary_graph) fn decode_for_wire_version(
    bytes: &[u8],
    wire_version: u16,
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<FactPayload, fact_decode_denial::FactDecodeDenial> {
    let mut decoder = FactDecoder::new(bytes, wire_version, request, allocation_policy)?;
    let mut facts = Vec::with_capacity(decoder.count());
    while let Some(fact) = decoder.next_fact()? {
        facts.push(fact);
    }
    Ok(FactPayload::decoded(facts, decoder.funding()))
}

fn put_locator(bytes: &mut impl FactBytes, locator: &AspectFieldLocator) -> Option<()> {
    bytes.push(match locator.aspect().authority() {
        LocatorAuthority::Authoritative => 1,
        LocatorAuthority::Derived => 2,
        LocatorAuthority::Projected => 3,
        LocatorAuthority::SupportOnly => 4,
        LocatorAuthority::Planned => 5,
        LocatorAuthority::ReceiptBearing => 6,
    });
    put_text(bytes, locator.aspect().aspect_key().as_str())?;
    let fields = locator.field_path().fields();
    if fields.is_empty() || fields.len() > MAXIMUM_FIELD_DEPTH {
        return None;
    }
    bytes.push(u8::try_from(fields.len()).ok()?);
    for field in fields {
        put_text(bytes, field.as_str())?;
    }
    Some(())
}

fn next_locator(cursor: &mut CheckpointCursor<'_>) -> Result<AspectFieldLocator, String> {
    let authority = match cursor.next_byte()? {
        1 => LocatorAuthority::Authoritative,
        2 => LocatorAuthority::Derived,
        3 => LocatorAuthority::Projected,
        4 => LocatorAuthority::SupportOnly,
        5 => LocatorAuthority::Planned,
        6 => LocatorAuthority::ReceiptBearing,
        _ => return Err("checkpoint fact locator authority is invalid".to_owned()),
    };
    let aspect = AspectKey::new(cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint fact aspect")?)
        .ok_or_else(|| "checkpoint fact aspect is invalid".to_owned())?;
    let field_count = usize::from(cursor.next_byte()?);
    if field_count == 0
        || field_count > MAXIMUM_FIELD_DEPTH
        // put_text and CheckpointCursor::next_bounded_text use u64 lengths.
        || field_count > cursor.remaining.len() / (std::mem::size_of::<u64>() + 1)
    {
        return Err("checkpoint fact field path length is invalid".to_owned());
    }
    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        fields.push(
            FieldKey::new(cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint fact field")?)
                .ok_or_else(|| "checkpoint fact field is invalid".to_owned())?,
        );
    }
    Ok(AspectFieldLocator::new(
        authority,
        aspect,
        CanonicalFieldPath::new(fields)
            .ok_or_else(|| "checkpoint fact field path is invalid".to_owned())?,
    ))
}

fn put_text(bytes: &mut impl FactBytes, text: &str) -> Option<()> {
    if text.is_empty() || text.len() > MAXIMUM_TEXT {
        return None;
    }
    put_u64(bytes, u64::try_from(text.len()).ok()?);
    bytes.extend_from_slice(text.as_bytes());
    Some(())
}

fn put_entity(bytes: &mut impl FactBytes, entity: EntityId) {
    put_u32(bytes, entity.partition_value());
    put_u64(bytes, entity.local_slot_value());
    put_u32(bytes, entity.generation_value());
}

fn put_optional_version(bytes: &mut impl FactBytes, version: Option<VersionId>) {
    match version {
        Some(version) => {
            bytes.push(1);
            put_u64(bytes, version.as_u64());
        }
        None => bytes.push(0),
    }
}

fn next_optional_version(cursor: &mut CheckpointCursor<'_>) -> Result<Option<VersionId>, String> {
    match cursor.next_byte()? {
        0 => Ok(None),
        1 => Ok(Some(VersionId(cursor.next_u64()?))),
        _ => Err("checkpoint fact revision posture is invalid".to_owned()),
    }
}

fn put_u32(bytes: &mut impl FactBytes, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
fn put_u64(bytes: &mut impl FactBytes, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests;
