//! Deterministic canonical binary encoding of Relational record payloads and
//! multi-version record history frames (zero `Debug` formatting, zero synthetic
//! placeholder strings).

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    FieldKey, InternedString, StructAspectValue,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{EntityId, VersionId};
use worth_relational::facade::publication::{PublishedAuthoritativePatch, RecordStructuralChange};

const ENTITY_PAYLOAD_MAGIC: &[u8; 5] = b"WHRE1";
const RELATION_PAYLOAD_MAGIC: &[u8; 5] = b"WHRR1";
const HISTORY_VERSION_MAGIC: &[u8; 5] = b"WHRV1";

/// Replayed authoritative aspect value reconstructed directly from a branch's
/// canonical ancestor commit patches (never querying `main`'s mirror!).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ReplayedAspectValue {
    Scalar(AspectValue),
    Struct(BTreeMap<FieldKey, AspectValue>),
}

/// Canonical aspect state map ordered by [`AspectKey`], matching
/// [`AuthoritativeRecordAspectState`]'s canonical iteration order.
pub(super) type ReplayedAspectState = BTreeMap<AspectKey, ReplayedAspectValue>;

/// Applies one committed canonical [`PublishedAuthoritativePatch`] onto
/// `state`, preserving untouched aspects and fields across ancestor commits.
pub(super) fn apply_published_patch(
    state: &mut ReplayedAspectState,
    patch: &PublishedAuthoritativePatch,
) {
    for aspect_key in patch.changed_aspects() {
        if patch.whole_clear_for(&aspect_key) {
            state.remove(&aspect_key);
            continue;
        }
        if let Some(scalar) = patch.scalar_set_for(&aspect_key) {
            state.insert(aspect_key, ReplayedAspectValue::Scalar(scalar.clone()));
            continue;
        }
        if let Some(struct_val) = patch.struct_set_for(&aspect_key) {
            let fields = struct_val
                .fields()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            state.insert(aspect_key, ReplayedAspectValue::Struct(fields));
            continue;
        }
        let mut fields = match state.remove(&aspect_key) {
            Some(ReplayedAspectValue::Struct(existing)) => existing,
            _ => BTreeMap::new(),
        };
        for cleared in patch.field_clears_for(&aspect_key) {
            fields.remove(cleared);
        }
        for set in patch.field_sets_for(&aspect_key) {
            fields.insert(set.field.clone(), set.value.clone());
        }
        state.insert(aspect_key, ReplayedAspectValue::Struct(fields));
    }
}

/// Canonically encodes a replayed entity aspect state with bit-identical layout
/// to [`encode_canonical_entity_payload`].
pub(super) fn encode_replayed_entity_payload(state: &ReplayedAspectState) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ENTITY_PAYLOAD_MAGIC);
    encode_replayed_aspect_state(&mut out, state);
    out
}

/// Canonically encodes a replayed relation endpoint + aspect state with
/// bit-identical layout to [`encode_canonical_relation_payload`].
pub(super) fn encode_replayed_relation_payload(
    source: EntityId,
    target: EntityId,
    state: &ReplayedAspectState,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(RELATION_PAYLOAD_MAGIC);
    encode_entity_id(&mut out, source);
    encode_entity_id(&mut out, target);
    encode_replayed_aspect_state(&mut out, state);
    out
}

fn encode_replayed_aspect_state(out: &mut Vec<u8>, state: &ReplayedAspectState) {
    out.extend_from_slice(&(state.len() as u32).to_le_bytes());
    for (aspect_key, value) in state {
        write_bytes(out, aspect_key.as_str().as_bytes());
        match value {
            ReplayedAspectValue::Scalar(scalar) => {
                out.push(1);
                encode_aspect_value(out, scalar);
            }
            ReplayedAspectValue::Struct(fields) => {
                out.push(2);
                out.extend_from_slice(&(fields.len() as u32).to_le_bytes());
                for (field_key, scalar) in fields {
                    write_bytes(out, field_key.as_str().as_bytes());
                    encode_aspect_value(out, scalar);
                }
            }
        }
    }
}

/// Canonically encodes an entity record's authoritative aspect payload bytes.
pub fn encode_canonical_entity_payload(
    aspect_state: Option<&AuthoritativeRecordAspectState>,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(ENTITY_PAYLOAD_MAGIC);
    encode_aspect_state(&mut out, aspect_state);
    out
}

/// Canonically encodes a relation record's endpoints and authoritative aspect payload bytes.
pub fn encode_canonical_relation_payload(
    source: EntityId,
    target: EntityId,
    aspect_state: Option<&AuthoritativeRecordAspectState>,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(RELATION_PAYLOAD_MAGIC);
    encode_entity_id(&mut out, source);
    encode_entity_id(&mut out, target);
    encode_aspect_state(&mut out, aspect_state);
    out
}

/// Canonically encodes one real committed historical version of a record
/// (commit, version, structural change, changed aspects, and the exact
/// canonical payload bytes observed at that historical version).
pub fn encode_canonical_history_version(
    commit_id: CommitId,
    version_id: VersionId,
    structural_change: RecordStructuralChange,
    changed_aspects: &[AspectKey],
    version_payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + version_payload.len());
    out.extend_from_slice(HISTORY_VERSION_MAGIC);
    out.extend_from_slice(&commit_id.0.to_le_bytes());
    out.extend_from_slice(&version_id.0.to_le_bytes());
    out.push(structural_change_tag(structural_change));
    out.extend_from_slice(&(changed_aspects.len() as u32).to_le_bytes());
    for aspect in changed_aspects {
        write_bytes(&mut out, aspect.as_str().as_bytes());
    }
    write_bytes(&mut out, version_payload);
    out
}

fn encode_aspect_state(out: &mut Vec<u8>, state: Option<&AuthoritativeRecordAspectState>) {
    let Some(state) = state else {
        out.extend_from_slice(&0u32.to_le_bytes());
        return;
    };
    let entries: Vec<_> = state.aspects().entries().collect();
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (aspect_key, value) in entries {
        write_bytes(out, aspect_key.as_str().as_bytes());
        encode_validated_value(out, value.view());
    }
}

fn encode_validated_value(out: &mut Vec<u8>, value: ContractValidatedAspectValueView<'_>) {
    match value {
        ContractValidatedAspectValueView::Scalar(scalar) => {
            out.push(1);
            encode_aspect_value(out, scalar);
        }
        ContractValidatedAspectValueView::Struct(struct_val) => {
            out.push(2);
            encode_struct_value(out, struct_val);
        }
    }
}

fn encode_struct_value(out: &mut Vec<u8>, value: &StructAspectValue) {
    let fields: Vec<_> = value.fields().collect();
    out.extend_from_slice(&(fields.len() as u32).to_le_bytes());
    for (field_key, scalar) in fields {
        write_bytes(out, field_key.as_str().as_bytes());
        encode_aspect_value(out, scalar);
    }
}

fn encode_aspect_value(out: &mut Vec<u8>, value: &AspectValue) {
    match value {
        AspectValue::Null => out.push(0),
        AspectValue::Bool(v) => {
            out.push(1);
            out.push(u8::from(*v));
        }
        AspectValue::Int8(v) => {
            out.push(2);
            out.push(*v as u8);
        }
        AspectValue::Int16(v) => {
            out.push(3);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::Int32(v) => {
            out.push(4);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::Int64(v) => {
            out.push(5);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::UInt8(v) => {
            out.push(6);
            out.push(*v);
        }
        AspectValue::UInt16(v) => {
            out.push(7);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::UInt32(v) => {
            out.push(8);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::UInt64(v) => {
            out.push(9);
            out.extend_from_slice(&v.to_le_bytes());
        }
        AspectValue::Float32(v) => {
            out.push(10);
            out.extend_from_slice(&v.bits().to_le_bytes());
        }
        AspectValue::Float64(v) => {
            out.push(11);
            out.extend_from_slice(&v.bits().to_le_bytes());
        }
        AspectValue::Decimal(v) => {
            out.push(12);
            write_bytes(out, v.as_str().as_bytes());
        }
        AspectValue::BigInt(v) => {
            out.push(13);
            write_bytes(out, v.as_str().as_bytes());
        }
        AspectValue::Rational(v) => {
            out.push(14);
            write_bytes(out, v.numerator.as_str().as_bytes());
            write_bytes(out, v.denominator.as_str().as_bytes());
        }
        AspectValue::String(s) => match s.raw_text() {
            Some(text) => {
                out.push(15);
                write_bytes(out, text.as_bytes());
            }
            None => match s {
                InternedString::Symbol(sym) => {
                    out.push(16);
                    out.extend_from_slice(&sym.0.to_le_bytes());
                }
                _ => {
                    out.push(15);
                    write_bytes(out, b"");
                }
            },
        },
        AspectValue::Uuid(u) => {
            out.push(17);
            out.extend_from_slice(u);
        }
        AspectValue::Date(d) => {
            out.push(18);
            out.extend_from_slice(&d.days_from_unix_epoch.to_le_bytes());
        }
        AspectValue::Time(t) => {
            out.push(19);
            out.extend_from_slice(&t.nanos_since_midnight.to_le_bytes());
        }
        AspectValue::Timestamp(ts) => {
            out.push(20);
            out.extend_from_slice(&ts.micros_since_unix_epoch.to_le_bytes());
        }
        AspectValue::TimestampTz(tz) => {
            out.push(21);
            out.extend_from_slice(&tz.utc_micros_since_unix_epoch.to_le_bytes());
            out.extend_from_slice(&tz.offset_minutes.to_le_bytes());
        }
        AspectValue::EntityRef(id) => {
            out.push(22);
            out.extend_from_slice(&id.partition_id.0.to_le_bytes());
            out.extend_from_slice(&id.local_slot.0.to_le_bytes());
            out.extend_from_slice(&id.generation.0.to_le_bytes());
        }
        AspectValue::ContentRef(id) => {
            out.push(23);
            out.extend_from_slice(&id.0);
        }
        AspectValue::InlineBytes(b) => {
            out.push(24);
            write_bytes(out, b.as_bytes());
        }
    }
}

fn encode_entity_id(out: &mut Vec<u8>, id: EntityId) {
    out.extend_from_slice(&id.partition_id.0.to_le_bytes());
    out.extend_from_slice(&id.local_slot.0.to_le_bytes());
    out.extend_from_slice(&id.generation.0.to_le_bytes());
}

const fn structural_change_tag(change: RecordStructuralChange) -> u8 {
    match change {
        RecordStructuralChange::Created => 1,
        RecordStructuralChange::Updated => 2,
        RecordStructuralChange::Deleted => 3,
        _ => 0,
    }
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}
