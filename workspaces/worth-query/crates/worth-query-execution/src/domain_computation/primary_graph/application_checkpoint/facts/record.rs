//! One current fact record; shared by eager checkpoint and streaming evidence reads.
use super::*;
pub(super) fn decode(
    cursor: &mut CheckpointCursor<'_>,
    control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
) -> Result<Fact, FactDecodeDenial> {
    Ok(match cursor.next_byte()? {
        6 => Fact::RetiredOutputEntity {
            entity_id: cursor.next_entity()?,
            kind: KindId(cursor.next_u32()?),
            created_at: VersionId(cursor.next_u64()?),
            deleted_at: VersionId(cursor.next_u64()?),
            read_locator: cursor
                .next_bounded_text(MAXIMUM_TEXT, "checkpoint retirement read locator")?,
        },
        1 => Fact::SourceEntity {
            entity_id: cursor.next_entity()?,
        },
        2 => {
            let entity_id = cursor.next_entity()?;
            let aspect =
                AspectKey::new(cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint fact aspect")?)
                    .ok_or_else(|| "checkpoint fact aspect is invalid".to_owned())?;
            let native_revision = next_optional_version(cursor)?.map(VersionId::as_u64);
            Fact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            }
        }
        3 => {
            let entity_id = cursor.next_entity()?;
            let locator = next_locator(cursor)?;
            let version = VersionId(cursor.next_u64()?);
            let presence = match cursor.next_byte()? {
                1 => RelationalFieldPresence::Present,
                2 => RelationalFieldPresence::Absent,
                _ => return Err("checkpoint field presence is invalid".to_owned().into()),
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
                _ => {
                    return Err("checkpoint adjacency direction is invalid"
                        .to_owned()
                        .into())
                }
            };
            let native_revision = next_optional_version(cursor)?;
            let comparison_work_limit = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint adjacency work limit exceeds host".to_owned())?;
            let endpoint_count = usize::try_from(cursor.next_u32()?)
                .map_err(|_| "checkpoint endpoint count exceeds host".to_owned())?;
            if endpoint_count > MAXIMUM_SET_ENTITIES || endpoint_count > cursor.remaining.len() / 16
            {
                return Err("checkpoint endpoint count is invalid".to_owned().into());
            }
            let mut endpoints = Vec::with_capacity(endpoint_count);
            for _ in 0..endpoint_count {
                control
                    .check_live()
                    .map_err(fact_decode_denial::FactDecodeDenial::Retention)?;
                endpoints.push(cursor.next_entity()?);
            }
            Fact::SourceAdjacencyRevision {
                    relation_kind,
                    anchor,
                    direction,
                    native_revision,
                    comparison_work_limit,
                    endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::preserve(
                        &endpoints,
                        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::control,
                    ).map_err(fact_decode_denial::FactDecodeDenial::Retention)?,
                }
        }
        5 => Fact::Entity {
            entity_id: cursor.next_entity()?,
            kind: KindId(cursor.next_u32()?),
        },
        7 => indexed_selection::decode(cursor)?,
        _ => {
            return Err("checkpoint producer fact kind is unsupported"
                .to_owned()
                .into())
        }
    })
}
