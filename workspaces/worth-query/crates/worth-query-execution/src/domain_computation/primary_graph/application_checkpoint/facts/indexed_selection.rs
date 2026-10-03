use std::sync::Arc;

use worth_relational::facade::{
    identity::KindId,
    indexes::{
        DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind, MAX_BOUNDED_INDEX_CANDIDATES,
    },
    storage::AuthoritativeFieldComparisonKey,
};

use super::{
    put_entity, put_locator, put_text, put_u32, put_u64, CheckpointCursor, Fact, MAXIMUM_FACTS,
    MAXIMUM_FACT_BYTES, MAXIMUM_TEXT,
};

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
        || *candidate_limit > MAX_BOUNDED_INDEX_CANDIDATES
        || candidates.len() > *candidate_limit
        || candidates.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return None;
    }
    let key_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)?;
    if key_bytes > MAXIMUM_FACT_BYTES as u64 {
        return None;
    }
    let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
    put_u64(bytes, index_id.0);
    put_text(bytes, &definition.name)?;
    bytes.push(u8::from(definition.branch_scoped));
    put_u32(bytes, entity_kind.as_u32());
    put_locator(bytes, locator)?;
    put_u32(
        bytes,
        u32::try_from(key.canonical_value_bytes().len()).ok()?,
    );
    bytes.extend_from_slice(key.canonical_value_bytes());
    put_u32(bytes, u32::try_from(*candidate_limit).ok()?);
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
    let locator = super::next_locator(cursor)?;
    let key_len = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint index key length exceeds host".to_owned())?;
    if key_len == 0 || key_len > MAXIMUM_FACT_BYTES {
        return Err("checkpoint index key length is invalid".to_owned());
    }
    let value =
        AuthoritativeFieldComparisonKey::value_from_canonical_bytes(cursor.next_bytes(key_len)?)
            .map_err(|denial| denial.to_string())?;
    let candidate_limit = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint index candidate limit exceeds host".to_owned())?;
    let count = usize::try_from(cursor.next_u32()?)
        .map_err(|_| "checkpoint index candidate count exceeds host".to_owned())?;
    if candidate_limit == 0
        || candidate_limit > MAX_BOUNDED_INDEX_CANDIDATES
        || count > candidate_limit
        || count > MAXIMUM_FACTS
        || count > cursor.remaining.len() / 16
    {
        return Err("checkpoint index candidate count is invalid".to_owned());
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(cursor.next_entity()?);
    }
    if candidates.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("checkpoint index candidates are not canonical".to_owned());
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
