use std::{io::Write, sync::Arc};

use worth_foundational::facade::AspectValue;
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};

use super::{
    next_locator, put_entity, put_locator, put_text, put_u32, put_u64, CheckpointCursor, Fact,
    KindId, MAXIMUM_SET_ENTITIES, MAXIMUM_TEXT,
};

const MAXIMUM_VALUE_BYTES: usize = 65_536;

pub(super) fn encode(bytes: &mut Vec<u8>, fact: &Fact) -> Option<()> {
    let Fact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates,
    } = fact
    else {
        return None;
    };
    if definition.index_id != *index_id
        || !matches!(&definition.kind, DerivedIndexKind::EntityField { field_locator } if field_locator == locator)
        || *candidate_limit == 0
        || *candidate_limit == usize::MAX
        || candidates.len() > *candidate_limit
        || candidates.len() > MAXIMUM_SET_ENTITIES
        || !candidates.windows(2).all(|pair| pair[0] < pair[1])
    {
        return None;
    }
    let mut encoded = ValueWriter(Vec::new());
    serde_json::to_writer(&mut encoded, value).ok()?;
    put_u64(bytes, index_id.0);
    put_text(bytes, &definition.name)?;
    bytes.push(u8::from(definition.branch_scoped));
    put_u32(bytes, entity_kind.as_u32());
    put_locator(bytes, locator)?;
    put_u32(bytes, u32::try_from(encoded.0.len()).ok()?);
    bytes.extend_from_slice(&encoded.0);
    put_u64(bytes, u64::try_from(*candidate_limit).ok()?);
    put_u32(bytes, u32::try_from(candidates.len()).ok()?);
    for candidate in candidates {
        put_entity(bytes, *candidate);
    }
    Some(())
}

pub(super) fn decode(cursor: &mut CheckpointCursor<'_>) -> Result<Fact, String> {
    let index_id = DerivedIndexId(cursor.next_u64()?);
    let name = cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint index definition name")?;
    let branch_scoped = match cursor.next_byte()? {
        0 => false,
        1 => true,
        _ => return Err("checkpoint index definition scope is invalid".to_owned()),
    };
    let entity_kind = KindId(cursor.next_u32()?);
    let locator = next_locator(cursor)?;
    let length = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint predicate value length exceeds host".to_owned())?;
    if length == 0 || length > MAXIMUM_VALUE_BYTES {
        return Err("checkpoint predicate value length is invalid".to_owned());
    }
    let value: AspectValue = serde_json::from_slice(cursor.next_bytes(length)?)
        .map_err(|_| "checkpoint predicate value is invalid".to_owned())?;
    let candidate_limit = usize::try_from(cursor.next_u64()?)
        .map_err(|_| "checkpoint predicate candidate limit exceeds host".to_owned())?;
    let count = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint predicate count exceeds host".to_owned())?;
    if candidate_limit == 0
        || candidate_limit == usize::MAX
        || count > candidate_limit
        || count > MAXIMUM_SET_ENTITIES
        || count > cursor.remaining.len() / 16
    {
        return Err("checkpoint predicate candidate count is invalid".to_owned());
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        let candidate = cursor.next_entity()?;
        if candidates
            .last()
            .is_some_and(|previous| *previous >= candidate)
        {
            return Err("checkpoint predicate candidates are not a unique ordered set".to_owned());
        }
        candidates.push(candidate);
    }
    let definition = Arc::new(DerivedIndexDefinition {
        index_id,
        name,
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped,
    });
    Ok(Fact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates,
    })
}

struct ValueWriter(Vec<u8>);
impl Write for ValueWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAXIMUM_VALUE_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "predicate value exceeds checkpoint budget",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
    };
    use worth_relational::facade::identity::{EntityId, PartitionId};

    fn selection(candidates: Vec<EntityId>) -> Fact {
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("predicate").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("value").unwrap()),
        );
        Fact::IndexedEntitySelection {
            index_id: DerivedIndexId(4),
            definition: Arc::new(DerivedIndexDefinition {
                index_id: DerivedIndexId(4),
                name: "predicate-index".to_owned(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: locator.clone(),
                },
                branch_scoped: false,
            }),
            entity_kind: KindId(2),
            locator,
            value: AspectValue::UInt64(u64::MAX),
            candidate_limit: 100,
            candidates,
        }
    }

    #[test]
    fn version_seven_preserves_absence_sets_and_exact_scalar_values() {
        for candidates in [vec![], vec![EntityId::new(PartitionId(0), 2, 1)]] {
            let fact = selection(candidates);
            let bytes = super::super::encode(std::slice::from_ref(&fact)).unwrap();
            assert_eq!(
                super::super::decode(
                    &bytes,
                    None,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation
                )
                .unwrap()
                .as_ref(),
                &[fact]
            );
            assert!(
                super::super::decode_for_wire_version(
                    &bytes,
                    6,
                    None,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation
                )
                .is_err(),
                "older checkpoint versions cannot reinterpret the new fact tag"
            );
        }
    }

    #[test]
    fn forged_duplicate_membership_is_denied_on_decode() {
        let fact = selection(vec![
            EntityId::new(PartitionId(0), 2, 1),
            EntityId::new(PartitionId(0), 3, 1),
        ]);
        let mut bytes = super::super::encode(&[fact]).unwrap();
        let length = bytes.len();
        let first = bytes[length - 32..length - 16].to_vec();
        bytes[length - 16..].copy_from_slice(&first);
        assert!(matches!(
            super::super::decode(
                &bytes,
                None,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            ).unwrap_err(),
            super::super::fact_decode_denial::FactDecodeDenial::Format(message)
                if message.contains("unique ordered set")
        ));
    }

    #[test]
    fn value_serialization_stops_at_the_declared_codec_budget() {
        let mut writer = ValueWriter(Vec::new());
        assert!(serde_json::to_writer(&mut writer, &"x".repeat(MAXIMUM_VALUE_BYTES + 1)).is_err());
        assert!(writer.0.len() <= MAXIMUM_VALUE_BYTES);
    }

    #[test]
    fn total_fact_capacity_does_not_widen_indexed_membership() {
        let mut fact = selection(
            (1..=MAXIMUM_SET_ENTITIES + 1)
                .map(|slot| EntityId::new(PartitionId(0), slot as u64, 1))
                .collect(),
        );
        let Fact::IndexedEntitySelection {
            candidate_limit, ..
        } = &mut fact
        else {
            unreachable!();
        };
        *candidate_limit = MAXIMUM_SET_ENTITIES + 1;
        assert!(super::super::encode(&[fact]).is_none());
    }
}
