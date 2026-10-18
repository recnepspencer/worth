//! Address correspondence does not confuse values, presence or generations.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    mvcc::CompanionPreflightBudget,
};

#[test]
fn changed_addresses_and_unsupported_retirements_cannot_share_a_projection() {
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 100_000,
        maximum_preparation_bytes: 1024 * 1024,
    });
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let old = Fact::SourceEntity { entity_id: entity };
    assert!(same_addresses(&old, &old, &mut admission).unwrap());
    let replacement = Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), 1, 2),
    };
    assert!(
        !same_addresses(&old, &replacement, &mut admission).unwrap(),
        "slot reuse is a different native address"
    );
    let retirement = Fact::RetiredOutputEntity {
        entity_id: entity,
        kind: worth_relational::facade::identity::KindId(14),
        created_at: worth_relational::facade::identity::VersionId(5),
        deleted_at: worth_relational::facade::identity::VersionId(15),
        read_locator: "native retirement".into(),
    };
    assert!(
        !same_addresses(&retirement, &retirement, &mut admission).unwrap(),
        "an unsupported projection is never an empty reusable one"
    );
}
