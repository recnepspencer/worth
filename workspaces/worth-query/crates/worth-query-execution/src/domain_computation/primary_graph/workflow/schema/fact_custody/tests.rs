use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    symbols::ClientKey,
    transactions::{CreatedEntityRef, EntityReference},
};

use super::WorkflowFactCustody;

#[test]
fn only_a_new_approval_or_successor_may_link_to_an_existing_fact() {
    let rule = WorkflowFactCustody {
        instance_kind: KindId(20),
        started_revision:
            crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                "workflow-instance",
                "started-program-revision",
            )
            .unwrap(),
        entity_kinds: [KindId(1), KindId(2), KindId(3), KindId(21), KindId(22)],
        relation_kinds: [
            KindId(4),
            KindId(5),
            KindId(6),
            KindId(7),
            KindId(8),
            KindId(9),
            KindId(10),
            KindId(11),
            KindId(12),
            KindId(13),
            KindId(14),
            KindId(15),
            KindId(16),
        ],
        existing_target_links: [
            (KindId(22), KindId(16)),
            (KindId(3), KindId(10)),
            (KindId(3), KindId(11)),
            (KindId(20), KindId(12)),
            (KindId(20), KindId(13)),
            (KindId(20), KindId(14)),
        ],
    };
    let created = |kind_id| {
        EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id,
            client_key: ClientKey::raw("custody-relation-test"),
        })
    };
    let existing = EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 1));
    assert!(rule.allows_relation_create(KindId(16), &created(KindId(22)), &existing));
    assert!(!rule.allows_relation_create(KindId(15), &created(KindId(21)), &existing));
    assert!(!rule.allows_relation_create(KindId(16), &existing, &existing));
    assert!(!rule.allows_relation_create(KindId(15), &existing, &created(KindId(22))));
    assert!(rule.allows_relation_create(KindId(10), &created(KindId(3)), &existing));
    assert!(rule.allows_relation_create(KindId(11), &created(KindId(3)), &existing));
    assert!(rule.allows_relation_create(KindId(9), &created(KindId(1)), &created(KindId(3))));
    assert!(!rule.allows_relation_create(KindId(10), &created(KindId(1)), &existing));
    assert!(!rule.allows_relation_create(KindId(9), &created(KindId(3)), &existing));
    assert!(!rule.allows_relation_create(KindId(10), &existing, &created(KindId(3))));
    assert!(!rule.allows_relation_create(KindId(10), &existing, &existing));
    assert!(rule.allows_relation_create(KindId(12), &created(KindId(20)), &existing));
    assert!(rule.allows_relation_create(KindId(13), &created(KindId(20)), &existing));
    assert!(rule.allows_relation_create(KindId(14), &created(KindId(20)), &existing));
    assert!(!rule.allows_relation_create(KindId(14), &created(KindId(3)), &existing));
    assert!(!rule.allows_relation_create(KindId(14), &existing, &existing));
    assert!(!rule.allows_relation_create(KindId(13), &created(KindId(3)), &existing));
    assert!(!rule.allows_relation_create(KindId(10), &created(KindId(20)), &existing));
    assert!(!rule.allows_relation_create(KindId(13), &existing, &existing));
}
