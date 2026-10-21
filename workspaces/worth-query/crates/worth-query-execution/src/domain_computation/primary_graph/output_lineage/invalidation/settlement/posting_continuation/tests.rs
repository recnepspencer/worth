//! Address correspondence does not confuse values, presence or generations.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;

#[test]
fn changed_addresses_and_unsupported_retirements_cannot_share_a_projection() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let old = Fact::SourceEntity { entity_id: entity };
    assert!(same_addresses(&old, &old).unwrap());
    let replacement = Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), 1, 2),
    };
    assert!(
        !same_addresses(&old, &replacement).unwrap(),
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
        !same_addresses(&retirement, &retirement).unwrap(),
        "an unsupported projection is never an empty reusable one"
    );
}
