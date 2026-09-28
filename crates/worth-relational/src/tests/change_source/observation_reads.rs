use crate::facade::change_source::RelationalObservationReadDenial;
use crate::facade::identity::{EntityId, RelationId};
use crate::facade::mvcc::RelationalBranchObservation;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{create_entity, create_relation, runtime_with_test_schema};

use super::observe;

/// An owner runtime holding one relation between two entities, its
/// observation of main, and a second runtime that did not issue it.
struct ForeignRead {
    owner: RelationalRuntime,
    other: RelationalRuntime,
    observation: RelationalBranchObservation,
    entity: EntityId,
    relation: RelationId,
}

fn foreign_read() -> ForeignRead {
    let owner = runtime_with_test_schema();
    let entity = create_entity(&owner, "source");
    let target = create_entity(&owner, "target");
    let relation = create_relation(&owner, entity, target, "link");
    let observation = observe(&owner, "main").observation();
    ForeignRead {
        owner,
        other: runtime_with_test_schema(),
        observation,
        entity,
        relation,
    }
}

#[test]
fn an_exact_entity_read_refuses_a_foreign_observation() {
    let read = foreign_read();

    assert!(read
        .owner
        .entity_record_at_observation(&read.observation, read.entity)
        .unwrap()
        .is_some());
    assert_eq!(
        read.other
            .entity_record_at_observation(&read.observation, read.entity)
            .unwrap_err(),
        RelationalObservationReadDenial::ForeignObservation
    );
}

#[test]
fn an_exact_relation_read_refuses_a_foreign_observation() {
    let read = foreign_read();

    assert!(read
        .owner
        .relation_record_at_observation(&read.observation, read.relation)
        .unwrap()
        .is_some());
    assert_eq!(
        read.other
            .relation_record_at_observation(&read.observation, read.relation)
            .unwrap_err(),
        RelationalObservationReadDenial::ForeignObservation
    );
}

#[test]
fn a_record_history_read_refuses_a_foreign_observation() {
    let read = foreign_read();

    assert!(read
        .owner
        .record_history_at_observation(&read.observation, read.entity)
        .unwrap()
        .is_some());
    assert_eq!(
        read.other
            .record_history_at_observation(&read.observation, read.entity)
            .unwrap_err(),
        RelationalObservationReadDenial::ForeignObservation
    );
}

#[test]
fn a_lineage_visibility_read_refuses_a_foreign_observation() {
    let read = foreign_read();
    let lineages = read
        .owner
        .record_history_at_observation(&read.observation, read.entity)
        .unwrap()
        .expect("the owned entity has a lineage")
        .resolved;

    assert_eq!(
        read.owner
            .visible_entities_for_lineages_at_observation(&read.observation, &lineages)
            .unwrap(),
        vec![read.entity]
    );
    assert_eq!(
        read.other
            .visible_entities_for_lineages_at_observation(&read.observation, &lineages)
            .unwrap_err(),
        RelationalObservationReadDenial::ForeignObservation
    );
}
