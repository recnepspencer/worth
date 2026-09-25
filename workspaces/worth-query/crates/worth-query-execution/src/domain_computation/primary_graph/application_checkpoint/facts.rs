//! Bounded v5 wire for the complete, postcommit-rebased producer fact set.
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

pub(super) const MAXIMUM_FACT_BYTES: usize = 1024 * 1024;
const MAXIMUM_FACTS: usize = 4096;
const MAXIMUM_TEXT: usize = 4096;
const MAXIMUM_FIELD_DEPTH: usize = 32;

pub(in crate::domain_computation::primary_graph) fn encode(facts: &[Fact]) -> Option<Vec<u8>> {
    if facts.is_empty() || facts.len() > MAXIMUM_FACTS {
        return None;
    }
    let mut bytes = Vec::new();
    put_u32(&mut bytes, u32::try_from(facts.len()).ok()?);
    for fact in facts {
        match fact {
            Fact::SourceEntity { entity_id } => {
                bytes.push(1);
                put_entity(&mut bytes, *entity_id);
            }
            Fact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => {
                bytes.push(2);
                put_entity(&mut bytes, *entity_id);
                put_text(&mut bytes, aspect.as_str())?;
                put_optional_version(&mut bytes, native_revision.map(VersionId));
            }
            Fact::SourceFieldRevision {
                entity_id,
                locator,
                native_revision,
            } => {
                let revision = native_revision.as_ref()?;
                bytes.push(3);
                put_entity(&mut bytes, *entity_id);
                put_locator(&mut bytes, locator)?;
                put_u64(&mut bytes, revision.version().as_u64());
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
                if endpoints.len() > MAXIMUM_FACTS {
                    return None;
                }
                bytes.push(4);
                put_u32(&mut bytes, relation_kind.as_u32());
                put_entity(&mut bytes, *anchor);
                bytes.push(match direction {
                    RelationalAdjacencyDirection::Outgoing => 1,
                    RelationalAdjacencyDirection::Incoming => 2,
                });
                put_optional_version(&mut bytes, *native_revision);
                put_u64(&mut bytes, u64::try_from(*comparison_work_limit).ok()?);
                put_u32(&mut bytes, u32::try_from(endpoints.len()).ok()?);
                for endpoint in endpoints {
                    put_entity(&mut bytes, *endpoint);
                }
            }
            Fact::Entity { entity_id, kind } => {
                bytes.push(5);
                put_entity(&mut bytes, *entity_id);
                put_u32(&mut bytes, kind.as_u32());
            }
            _ => return None,
        }
        if bytes.len() > MAXIMUM_FACT_BYTES {
            return None;
        }
    }
    Some(bytes)
}

pub(in crate::domain_computation::primary_graph) fn decode(
    bytes: &[u8],
) -> Result<Arc<[Fact]>, String> {
    if bytes.len() < 5 || bytes.len() > MAXIMUM_FACT_BYTES {
        return Err("checkpoint producer fact payload length is invalid".to_owned());
    }
    let mut cursor = CheckpointCursor::new(bytes);
    let count = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint producer fact count exceeds host".to_owned())?;
    if count == 0 || count > MAXIMUM_FACTS || count > bytes.len().saturating_sub(4) {
        return Err("checkpoint producer fact count is invalid".to_owned());
    }
    let mut facts = Vec::with_capacity(count);
    for _ in 0..count {
        let fact = match cursor.next_byte()? {
            1 => Fact::SourceEntity {
                entity_id: cursor.next_entity()?,
            },
            2 => Fact::SourceAspectRevision {
                entity_id: cursor.next_entity()?,
                aspect: AspectKey::new(
                    cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint fact aspect")?,
                )
                .ok_or_else(|| "checkpoint fact aspect is invalid".to_owned())?,
                native_revision: next_optional_version(&mut cursor)?.map(VersionId::as_u64),
            },
            3 => {
                let entity_id = cursor.next_entity()?;
                let locator = next_locator(&mut cursor)?;
                let version = VersionId(cursor.next_u64()?);
                let presence = match cursor.next_byte()? {
                    1 => RelationalFieldPresence::Present,
                    2 => RelationalFieldPresence::Absent,
                    _ => return Err("checkpoint field presence is invalid".to_owned()),
                };
                Fact::SourceFieldRevision {
                    entity_id,
                    locator,
                    native_revision: Some(RelationalFieldRevision::new(version, presence)),
                }
            }
            4 => {
                let relation_kind = KindId(cursor.next_u32()?);
                let anchor = cursor.next_entity()?;
                let direction = match cursor.next_byte()? {
                    1 => RelationalAdjacencyDirection::Outgoing,
                    2 => RelationalAdjacencyDirection::Incoming,
                    _ => return Err("checkpoint adjacency direction is invalid".to_owned()),
                };
                let native_revision = next_optional_version(&mut cursor)?;
                let comparison_work_limit = usize::try_from(cursor.next_u64()?)
                    .map_err(|_| "checkpoint adjacency work limit exceeds host".to_owned())?;
                let endpoint_count = usize::try_from(cursor.next_u32()?)
                    .map_err(|_| "checkpoint endpoint count exceeds host".to_owned())?;
                if endpoint_count > MAXIMUM_FACTS || endpoint_count > cursor.remaining.len() / 16 {
                    return Err("checkpoint endpoint count is invalid".to_owned());
                }
                let mut endpoints = Vec::with_capacity(endpoint_count);
                for _ in 0..endpoint_count {
                    endpoints.push(cursor.next_entity()?);
                }
                Fact::SourceAdjacencyRevision {
                    relation_kind,
                    anchor,
                    direction,
                    native_revision,
                    comparison_work_limit,
                    endpoints,
                }
            }
            5 => Fact::Entity {
                entity_id: cursor.next_entity()?,
                kind: KindId(cursor.next_u32()?),
            },
            _ => return Err("checkpoint producer fact kind is unsupported".to_owned()),
        };
        facts.push(fact);
    }
    if !cursor.is_empty() {
        return Err("checkpoint producer fact payload length differs".to_owned());
    }
    Ok(facts.into())
}

fn put_locator(bytes: &mut Vec<u8>, locator: &AspectFieldLocator) -> Option<()> {
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
        || field_count > cursor.remaining.len() / 9
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

fn put_text(bytes: &mut Vec<u8>, text: &str) -> Option<()> {
    if text.is_empty() || text.len() > MAXIMUM_TEXT {
        return None;
    }
    put_u64(bytes, u64::try_from(text.len()).ok()?);
    bytes.extend_from_slice(text.as_bytes());
    Some(())
}

fn put_entity(bytes: &mut Vec<u8>, entity: EntityId) {
    put_u32(bytes, entity.partition_value());
    put_u64(bytes, entity.local_slot_value());
    put_u32(bytes, entity.generation_value());
}

fn put_optional_version(bytes: &mut Vec<u8>, version: Option<VersionId>) {
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

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests;
