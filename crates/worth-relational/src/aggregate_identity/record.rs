//! The single scalar grammar of private aggregate identity records.
use std::collections::BTreeMap;
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AspectFieldLocator, AspectKey, AspectValue,
    CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
};

use super::AggregateLineageKind;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, PartitionId};

pub(crate) const AGGREGATE_ASPECT: &str = "aggregate-identity";
pub(crate) const LINEAGE_ASPECT: &str = "aggregate-lineage";
pub(crate) const AGGREGATE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("identity-high", ScalarAspectType::Uuid),
    ("identity-low", ScalarAspectType::Uuid),
    ("definition-high", ScalarAspectType::Uuid),
    ("definition-low", ScalarAspectType::Uuid),
    ("born", ScalarAspectType::UInt64),
    ("seed-partition", ScalarAspectType::UInt32),
    ("seed-slot", ScalarAspectType::UInt64),
    ("seed-generation", ScalarAspectType::UInt32),
    ("current-partition", ScalarAspectType::UInt32),
    ("current-slot", ScalarAspectType::UInt64),
    ("current-generation", ScalarAspectType::UInt32),
    ("active", ScalarAspectType::Bool),
    ("retired", ScalarAspectType::UInt64),
];
pub(crate) const LINEAGE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("occurrence", ScalarAspectType::UInt64),
    ("kind", ScalarAspectType::UInt8),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AggregateRecord {
    pub(crate) identity: AggregateIdentity,
    pub(crate) definition: AggregateDefinitionIdentity,
    pub(crate) born: CommitId,
    pub(crate) seed: EntityId,
    pub(crate) current_anchor: Option<EntityId>,
    pub(crate) retired: Option<CommitId>,
}

pub(crate) fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).expect("fixed aggregate aspect"),
        CanonicalFieldPath::single(FieldKey::new(field).expect("fixed aggregate field")),
    )
}

impl AggregateRecord {
    pub(crate) fn fields(&self) -> BTreeMap<AspectFieldLocator, AspectValue> {
        let mut fields = BTreeMap::new();
        put_digest(&mut fields, "identity", self.identity.bytes());
        put_digest(&mut fields, "definition", self.definition.bytes());
        fields.insert(
            locator(AGGREGATE_ASPECT, "born"),
            AspectValue::UInt64(self.born.0),
        );
        put_entity(&mut fields, "seed", self.seed);
        put_entity(
            &mut fields,
            "current",
            self.current_anchor.unwrap_or(self.seed),
        );
        fields.insert(
            locator(AGGREGATE_ASPECT, "active"),
            AspectValue::Bool(self.current_anchor.is_some()),
        );
        fields.insert(
            locator(AGGREGATE_ASPECT, "retired"),
            AspectValue::UInt64(self.retired.map_or(0, |at| at.0)),
        );
        fields
    }

    pub(crate) fn decode<'a>(
        mut read: impl FnMut(&AspectFieldLocator) -> Option<&'a AspectValue>,
    ) -> Option<Self> {
        let mut values = BTreeMap::new();
        for &(field, _) in AGGREGATE_FIELDS {
            values.insert(field, read(&locator(AGGREGATE_ASPECT, field))?);
        }
        let active = match values["active"] {
            AspectValue::Bool(value) => *value,
            _ => return None,
        };
        let current = decode_entity(&values, "current")?;
        Some(Self {
            identity: AggregateIdentity::from_bytes(decode_digest(&values, "identity")?),
            definition: AggregateDefinitionIdentity::from_bytes(decode_digest(
                &values,
                "definition",
            )?),
            born: CommitId(u64_value(values["born"])?),
            seed: decode_entity(&values, "seed")?,
            current_anchor: active.then_some(current),
            retired: (!active).then_some(CommitId(u64_value(values["retired"])?)),
        })
    }
}

pub(crate) fn lineage_fields(
    occurrence: CommitId,
    kind: AggregateLineageKind,
) -> BTreeMap<AspectFieldLocator, AspectValue> {
    BTreeMap::from([
        (
            locator(LINEAGE_ASPECT, "occurrence"),
            AspectValue::UInt64(occurrence.0),
        ),
        (
            locator(LINEAGE_ASPECT, "kind"),
            AspectValue::UInt8(match kind {
                AggregateLineageKind::SplitFrom => 1,
                AggregateLineageKind::MergedInto => 2,
            }),
        ),
    ])
}

fn put_digest(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    prefix: &str,
    bytes: &[u8; 32],
) {
    for (suffix, half) in [("high", &bytes[..16]), ("low", &bytes[16..])] {
        fields.insert(
            locator(AGGREGATE_ASPECT, &format!("{prefix}-{suffix}")),
            AspectValue::Uuid(half.try_into().expect("digest half")),
        );
    }
}

fn put_entity(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    prefix: &str,
    entity: EntityId,
) {
    for (suffix, value) in [
        ("partition", AspectValue::UInt32(entity.partition_value())),
        ("slot", AspectValue::UInt64(entity.local_slot_value())),
        ("generation", AspectValue::UInt32(entity.generation_value())),
    ] {
        fields.insert(
            locator(AGGREGATE_ASPECT, &format!("{prefix}-{suffix}")),
            value,
        );
    }
}

fn decode_digest(values: &BTreeMap<&str, &AspectValue>, prefix: &str) -> Option<[u8; 32]> {
    let mut bytes = [0; 32];
    let (high, low) = bytes.split_at_mut(16);
    for (suffix, output) in [("high", high), ("low", low)] {
        let AspectValue::Uuid(half) = values.get(format!("{prefix}-{suffix}").as_str())? else {
            return None;
        };
        output.copy_from_slice(half);
    }
    Some(bytes)
}

fn decode_entity(values: &BTreeMap<&str, &AspectValue>, prefix: &str) -> Option<EntityId> {
    let AspectValue::UInt32(partition) = values.get(format!("{prefix}-partition").as_str())? else {
        return None;
    };
    let slot = u64_value(values.get(format!("{prefix}-slot").as_str())?)?;
    let AspectValue::UInt32(generation) = values.get(format!("{prefix}-generation").as_str())?
    else {
        return None;
    };
    Some(EntityId::new(
        PartitionId::new(*partition),
        slot,
        *generation,
    ))
}

fn u64_value(value: &AspectValue) -> Option<u64> {
    match value {
        AspectValue::UInt64(value) => Some(*value),
        _ => None,
    }
}
