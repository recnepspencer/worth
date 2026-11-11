//! Pinned persistent index containers and canonical value owners before decode.
use super::*;
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::*;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use std::sync::Arc;

pub(in super::super) fn quote(
    value: BorrowedPayload<'_>,
    request: &dyn PayloadCheckpoint,
) -> Result<u64, Refusal> {
    let mut bytes = 0;
    let mut variants = 0;
    value.fields(request, |name, value| {
        variants += 1;
        let addition = match name {
            "EntityField" => map::<AuthoritativeFieldComparisonKey, EntityId>(
                value, request, comparison, scalar,
            )?,
            "RelationField" => map::<AuthoritativeFieldComparisonKey, RelationId>(
                value, request, comparison, scalar,
            )?,
            "RelatedEntityOrdering" => {
                map::<EntityId, RelatedEntityOrderingEntry>(value, request, scalar, ordering)?
            }
            "RelationJoin" => {
                // The join index is its row map plus the leg state that maintains it.
                let mut bytes = map::<RelationJoinKey, RelationJoinEntry>(
                    value.field("entries", request)?,
                    request,
                    scalar,
                    scalar,
                )?;
                add(&mut bytes, legs(value.field("legs", request)?, request)?)?;
                bytes
            }
            _ => return Err(malformed()),
        };
        add(&mut bytes, addition)
    })?;
    if variants != 1 {
        return Err(malformed());
    }
    Ok(bytes)
}
fn map<K: Ord + Clone, R: Clone>(
    value: BorrowedPayload<'_>,
    request: &dyn PayloadCheckpoint,
    key_quote: fn(BorrowedPayload<'_>, &dyn PayloadCheckpoint) -> Result<u64, Refusal>,
    row_quote: fn(BorrowedPayload<'_>, &dyn PayloadCheckpoint) -> Result<u64, Refusal>,
) -> Result<u64, Refusal> {
    let mut bytes = 0;
    let count = value.pairs(request, |key, rows| {
        add(&mut bytes, shared::<K>()?)?;
        add(&mut bytes, key_quote(key, request)?)?;
        let count = rows.elements(request, |row| {
            add(&mut bytes, shared::<R>()?)?;
            add(&mut bytes, row_quote(row, request)?)
        })?;
        // im 15.1 Serde materializes Vec<Arc<R>> before its persistent vector.
        add(&mut bytes, vector::<Arc<R>>(count)?)?;
        let (_, backing) =
            crate::indexes::row_copy_bounds::<R>(count, true).map_err(|_| malformed())?;
        add(&mut bytes, backing as u64)
    })?;
    // Its map visitor likewise retains a concrete Vec of key/row handles.
    add(&mut bytes, vector::<(Arc<K>, DerivedIndexRows<R>)>(count)?)?;
    let (path, node) =
        crate::indexes::persistent_map_path_bytes::<K, R>(count).map_err(|_| malformed())?;
    let nodes = count
        .div_ceil(31)
        .checked_add(2)
        .and_then(|n| n.checked_add(path))
        .ok_or_else(malformed)?;
    add(
        &mut bytes,
        u64::try_from(nodes.checked_mul(node).ok_or_else(malformed)?).map_err(|_| malformed())?,
    )?;
    Ok(bytes)
}
/// Every persistent container of both legs is priced as decode builds it:
/// im Serde's intermediate Vec plus its node chunks, per outer map, inner map,
/// relation set and inverse map. The owner's retained-size model is not a bound.
fn legs(value: BorrowedPayload<'_>, request: &dyn PayloadCheckpoint) -> Result<u64, Refusal> {
    if value.is_none() {
        return Ok(0);
    }
    let mut bytes = shared::<JoinLegsState>()?;
    for side in ["left", "right"] {
        let leg = value.field(side, request)?;
        let outer = leg
            .field("shared", request)?
            .pairs(request, |_, externals| {
                let inner = externals.pairs(request, |_, rows| {
                    let relations = rows.elements(request, |_| Ok(()))?;
                    add(&mut bytes, container::<RelationId, ()>(relations)?)
                })?;
                add(
                    &mut bytes,
                    container::<EntityId, im::OrdSet<RelationId>>(inner)?,
                )
            })?;
        add(
            &mut bytes,
            container::<EntityId, im::OrdMap<EntityId, im::OrdSet<RelationId>>>(outer)?,
        )?;
        let inverse = leg
            .field("relations", request)?
            .pairs(request, |_, _| Ok(()))?;
        add(
            &mut bytes,
            container::<RelationId, (EntityId, EntityId)>(inverse)?,
        )?;
    }
    Ok(bytes)
}
/// One decoded im container: Serde's intermediate Vec of its entries plus
/// 64-slot node chunks (31-entry minimum occupancy, root and split slack).
fn container<K, V>(count: usize) -> Result<u64, Refusal> {
    let node = std::mem::size_of::<(K, V)>()
        .checked_mul(64)
        .and_then(|n| n.checked_add(std::mem::size_of::<usize>() * 71))
        .ok_or_else(malformed)?;
    let nodes = count.div_ceil(31).checked_add(2).ok_or_else(malformed)?;
    let mut bytes = vector::<(K, V)>(count)?;
    add(
        &mut bytes,
        u64::try_from(nodes.checked_mul(node).ok_or_else(malformed)?).map_err(|_| malformed())?,
    )?;
    Ok(bytes)
}
fn scalar(_: BorrowedPayload<'_>, _: &dyn PayloadCheckpoint) -> Result<u64, Refusal> {
    Ok(0)
}
fn comparison(value: BorrowedPayload<'_>, request: &dyn PayloadCheckpoint) -> Result<u64, Refusal> {
    let raw = value.field("canonical_value_bytes", request)?;
    let mut octets = raw.octets(request)?;
    let count = octets.len();
    octets.skip(count)?;
    octets.finish()?;
    vector::<u8>(count)
}
fn ordering(value: BorrowedPayload<'_>, request: &dyn PayloadCheckpoint) -> Result<u64, Refusal> {
    let mut bytes = 0;
    let count = value
        .field("ordering_values", request)?
        .elements(request, |value| {
            let mut octets = value.field("value", request)?.octets(request)?;
            let count = octets.len();
            let nested =
                crate::aspect_wire::value_frame_decoded_bytes(count, || octets.byte(), malformed)?;
            octets.finish()?;
            add(&mut bytes, nested.retained_bytes)?;
            add(&mut bytes, nested.transient_bytes)?;
            add(&mut bytes, vector::<u8>(count)?)
        })?;
    add(&mut bytes, vector::<RelatedEntityOrderingValue>(count)?)?;
    Ok(bytes)
}
