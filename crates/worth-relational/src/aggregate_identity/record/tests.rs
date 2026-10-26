//! Fixed wire grammar controls; no malformed graph or mutation authority is forged.
use super::*;

#[test]
fn lifecycle_wire_fields_and_closed_lineage_tags_refuse_contradictions() {
    let mut fields = BTreeMap::from([
        (
            locator(AGGREGATE_ASPECT, "identity-high"),
            AspectValue::Uuid([1; 16]),
        ),
        (
            locator(AGGREGATE_ASPECT, "identity-low"),
            AspectValue::Uuid([1; 16]),
        ),
        (
            locator(AGGREGATE_ASPECT, "definition-high"),
            AspectValue::Uuid([2; 16]),
        ),
        (
            locator(AGGREGATE_ASPECT, "definition-low"),
            AspectValue::Uuid([2; 16]),
        ),
        (locator(AGGREGATE_ASPECT, "born"), AspectValue::UInt64(7)),
        (
            locator(AGGREGATE_ASPECT, "seed-partition"),
            AspectValue::UInt32(11),
        ),
        (
            locator(AGGREGATE_ASPECT, "seed-slot"),
            AspectValue::UInt64(7),
        ),
        (
            locator(AGGREGATE_ASPECT, "seed-generation"),
            AspectValue::UInt32(4),
        ),
        (
            locator(AGGREGATE_ASPECT, "current-partition"),
            AspectValue::UInt32(11),
        ),
        (
            locator(AGGREGATE_ASPECT, "current-slot"),
            AspectValue::UInt64(12),
        ),
        (
            locator(AGGREGATE_ASPECT, "current-generation"),
            AspectValue::UInt32(5),
        ),
        (locator(AGGREGATE_ASPECT, "active"), AspectValue::Bool(true)),
        (locator(AGGREGATE_ASPECT, "retired"), AspectValue::UInt64(0)),
    ]);
    let seed = EntityId::new(PartitionId::new(11), 7, 4);
    let active = AggregateRecord::decode(|key| fields.get(key)).unwrap();
    assert_eq!(active.identity, AggregateIdentity::from_bytes([1; 32]));
    assert_eq!(
        active.definition,
        AggregateDefinitionIdentity::from_bytes([2; 32])
    );
    assert_eq!(active.seed, seed);
    assert_eq!(
        active.current_anchor,
        Some(EntityId::new(PartitionId::new(11), 12, 5))
    );
    assert_eq!(active.retired, None);
    fields.insert(
        locator(AGGREGATE_ASPECT, "retired"),
        AspectValue::UInt64(29),
    );
    assert!(
        AggregateRecord::decode(|key| fields.get(key)).is_none(),
        "active cannot conceal retirement"
    );
    fields.insert(
        locator(AGGREGATE_ASPECT, "active"),
        AspectValue::Bool(false),
    );
    assert!(
        AggregateRecord::decode(|key| fields.get(key)).is_none(),
        "retired current coordinates must be the writer's seed sentinel"
    );
    fields.insert(
        locator(AGGREGATE_ASPECT, "current-slot"),
        AspectValue::UInt64(7),
    );
    fields.insert(
        locator(AGGREGATE_ASPECT, "current-generation"),
        AspectValue::UInt32(4),
    );
    let retired = AggregateRecord::decode(|key| fields.get(key)).unwrap();
    assert_eq!(retired.current_anchor, None);
    assert_eq!(retired.retired, Some(CommitId(29)));
    fields.insert(locator(AGGREGATE_ASPECT, "retired"), AspectValue::UInt64(0));
    assert!(
        AggregateRecord::decode(|key| fields.get(key)).is_none(),
        "zero is the no-retirement sentinel, not a committed retirement"
    );
    for (tag, expected) in [
        (1, Some(AggregateLineageKind::SplitFrom)),
        (2, Some(AggregateLineageKind::MergedInto)),
        (0, None),
        (255, None),
    ] {
        let lineage = BTreeMap::from([
            (
                locator(LINEAGE_ASPECT, "occurrence"),
                AspectValue::UInt64(29),
            ),
            (locator(LINEAGE_ASPECT, "kind"), AspectValue::UInt8(tag)),
        ]);
        let decoded = LineageRecord::decode(|key| lineage.get(key));
        assert_eq!(decoded.map(|record| record.kind), expected);
        assert!(decoded.is_none_or(|record| record.occurrence == CommitId(29)));
    }
}
