//! Original binding identities remain observable even when no response escaped.
use super::{
    comparison::{HostDivergence, ProjectionPart},
    projection::StateProjection,
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalDigestId, ContractValidatedAspectValueView, FieldKey,
};
use worth_relational::facade::{
    identity::{EntityId, VersionId},
    runtime::EntityReadRecord,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct IdempotencyProjection {
    pub entity: EntityId,
    pub created: VersionId,
    pub key: CanonicalDigestId,
    pub intent: Vec<(String, Vec<CanonicalDigestId>)>,
    pub outcome: u64,
    pub emitted: u64,
    pub settlement: u64,
}
fn value<'a>(row: &'a EntityReadRecord, name: &str) -> &'a AspectValue {
    let value = row
        .authoritative_aspect_state
        .as_ref()
        .expect("actual binding state")
        .get(&AspectKey::new("idempotency").unwrap())
        .expect("actual idempotency aspect");
    let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
        panic!("actual idempotency contract is struct")
    };
    fields
        .get(&FieldKey::new(name).unwrap())
        .expect("actual registered idempotency field")
}
fn number(row: &EntityReadRecord, name: &str) -> u64 {
    let AspectValue::UInt64(value) = value(row, name) else {
        panic!("actual binding numeric field {name}")
    };
    *value
}
fn text(
    row: &EntityReadRecord,
    name: &str,
    symbols: &worth_relational::facade::symbols::SymbolTableSnapshot,
) -> String {
    let AspectValue::String(value) = value(row, name) else {
        panic!("actual binding text field {name}")
    };
    super::semantic_text::text(value, symbols)
}
fn digest(value: &str) -> CanonicalDigestId {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .expect("owner-recorded canonical digest uses its type-owned codec")
}
pub fn observe(
    rows: &[EntityReadRecord],
    symbols: &worth_relational::facade::symbols::SymbolTableSnapshot,
) -> Vec<IdempotencyProjection> {
    rows.iter()
        .filter(|row| row.kind.kind_name == "worth-query-provider-idempotency")
        .map(|row| {
            let intent = text(row, "intent", symbols);
            let slots = intent
                .strip_prefix("v2:")
                .expect("current owner intent encoding");
            let intent = slots
                .split(':')
                .enumerate()
                .map(|(index, slot)| {
                    let (name, value) = if index == 0 {
                        ("root", slot)
                    } else {
                        slot.split_once('=')
                            .expect("owner-recorded named intent component")
                    };
                    let words = if value == "-" {
                        Vec::new()
                    } else {
                        assert_eq!(
                            value.len() % 64,
                            0,
                            "owner-recorded digest or durable scope tuple"
                        );
                        value
                            .as_bytes()
                            .as_chunks::<64>()
                            .0
                            .iter()
                            .map(|word| digest(std::str::from_utf8(word).unwrap()))
                            .collect()
                    };
                    (name.into(), words)
                })
                .collect();
            IdempotencyProjection {
                entity: row.entity_id,
                created: row.created_at_version,
                key: digest(&text(row, "key", symbols)),
                intent,
                outcome: number(row, "outcome-identity"),
                emitted: number(row, "emitted-effect-count"),
                settlement: number(row, "output-settlement"),
            }
        })
        .collect()
}
fn pair(
    map: &mut DurableIdentityCorrespondence,
    h: &IdempotencyProjection,
    r: &IdempotencyProjection,
) -> Result<(), HostDivergence> {
    let bad = || HostDivergence::Meaning {
        part: ProjectionPart::Idempotency,
        index: 0,
    };
    if h.emitted != r.emitted || h.settlement != r.settlement || h.intent.len() != r.intent.len() {
        return Err(bad());
    }
    map.admit(Id::Entity(h.entity), Id::Entity(r.entity))?;
    map.admit(Id::NativeVersion(h.created), Id::NativeVersion(r.created))?;
    map.admit(Id::QueryOutcome(h.outcome), Id::QueryOutcome(r.outcome))?;
    if h.key != r.key || h.intent != r.intent {
        return Err(bad());
    }

    Ok(())
}
pub fn compare(
    map: &mut DurableIdentityCorrespondence,
    h: &StateProjection,
    r: &StateProjection,
) -> Result<(), HostDivergence> {
    let bad = || HostDivergence::Meaning {
        part: ProjectionPart::Idempotency,
        index: 0,
    };
    let unique_home: std::collections::HashSet<_> =
        h.idempotency.iter().map(|row| row.key).collect();
    let unique_reference: std::collections::HashSet<_> =
        r.idempotency.iter().map(|row| row.key).collect();
    if h.idempotency.len() != r.idempotency.len()
        || unique_home.len() != h.idempotency.len()
        || unique_reference.len() != r.idempotency.len()
    {
        return Err(bad());
    }
    for h in &h.idempotency {
        let r = r
            .idempotency
            .iter()
            .find(|r| r.key == h.key)
            .ok_or_else(bad)?;
        pair(map, h, r)?;
    }
    Ok(())
}
