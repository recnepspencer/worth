//! Additive presence-predicate encoding; existing fact tags keep their meaning.
use super::{put_entity, put_text, put_u32, CheckpointCursor, FactDecodeDenial, MAXIMUM_TEXT};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::KindId;

pub(super) fn encode(bytes: &mut Vec<u8>, fact: &Fact) -> Option<()> {
    let Fact::AspectPresence {
        entity_id,
        kind,
        aspect,
        present,
    } = fact
    else {
        return None;
    };
    put_entity(bytes, *entity_id);
    put_u32(bytes, kind.as_u32());
    put_text(bytes, aspect.as_str())?;
    bytes.push(u8::from(*present));
    Some(())
}

pub(super) fn decode(cursor: &mut CheckpointCursor<'_>) -> Result<Fact, FactDecodeDenial> {
    let entity_id = cursor.next_entity()?;
    let kind = KindId(cursor.next_u32()?);
    let aspect =
        AspectKey::new(cursor.next_bounded_text(MAXIMUM_TEXT, "checkpoint aspect presence")?)
            .ok_or_else(|| "checkpoint aspect presence key is invalid".to_owned())?;
    let present = match cursor.next_byte()? {
        0 => false,
        1 => true,
        _ => return Err("checkpoint aspect presence is invalid".to_owned().into()),
    };
    Ok(Fact::AspectPresence {
        entity_id,
        kind,
        aspect,
        present,
    })
}
