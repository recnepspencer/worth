use worth_foundational::facade::{AspectContractRevision, AspectKey};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, VersionId},
    runtime::RelationalAdjacencyDirection,
};

use super::{
    normalize_source_footprint, WorthQueryObservedAdjacencyRevision,
    WorthQueryObservedAspectRevision, WorthQueryObservedSourceFootprint,
};

#[test]
fn normalization_deduplicates_revisited_native_dependencies() {
    let entity = EntityId::new(PartitionId::main(), 7, 1);
    let aspect = WorthQueryObservedAspectRevision {
        entity,
        entity_name: "Body".to_owned(),
        aspect: AspectKey::new("body-facts").unwrap(),
        contract_revision: AspectContractRevision(1),
        native_revision: Some(11),
    };
    let incoming = adjacency(entity, RelationalAdjacencyDirection::Incoming);
    let outgoing = adjacency(entity, RelationalAdjacencyDirection::Outgoing);
    let mut footprint = WorthQueryObservedSourceFootprint {
        root: entity,
        complete: true,
        entities: vec![entity, entity],
        aspects: vec![aspect.clone(), aspect],
        adjacencies: vec![incoming.clone(), outgoing.clone(), incoming, outgoing],
        root_selection: None,
    };

    let retained_before = footprint.retained_bytes();
    let released_bytes = normalize_source_footprint(&mut footprint);

    assert_eq!(footprint.entities, vec![entity]);
    assert_eq!(footprint.aspects.len(), 1);
    assert_eq!(footprint.adjacencies.len(), 2);
    assert_eq!(released_bytes, retained_before - footprint.retained_bytes());
    assert!(released_bytes > 0);
}

fn adjacency(
    anchor: EntityId,
    direction: RelationalAdjacencyDirection,
) -> WorthQueryObservedAdjacencyRevision {
    WorthQueryObservedAdjacencyRevision {
        anchor,
        relation_kind: KindId::new(19),
        direction,
        native_revision: Some(VersionId(23)),
        comparison_work_limit: 4,
        endpoints: vec![EntityId::new(PartitionId::main(), 8, 1)],
    }
}
