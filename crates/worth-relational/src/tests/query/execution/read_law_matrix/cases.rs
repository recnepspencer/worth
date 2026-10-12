use super::*;
use work::Quote;

pub(super) struct Case {
    pub name: &'static str,
    pub scope: QueryScope,
    pub ordering: QueryOrderingContract,
    pub entities: &'static [usize],
    pub relations: &'static [usize],
    pub work: Quote,
}

// A one-partition fixture: three named vertices and the chain a -> b -> c.
// "a" fails the predicate on two rows; "absent" fails on all three.
// Each small record and each captured fragment fits one 4096-byte work block.
pub(super) fn table(world: &fixture::World) -> Vec<Case> {
    let partition = Some(Arc::from([PartitionId(7)]));
    let entity_order = QueryOrderingContract::CanonicalEntityIdOrder;
    let relation_order = QueryOrderingContract::CanonicalRelationIdOrder;
    let mut cases = vec![
        Case {
            name: "explicit-mixed",
            scope: QueryScope::ExplicitTargets {
                targets: Arc::from([
                    RecordRef::Entity(world.entities[0]),
                    RecordRef::Relation(world.relations[0]),
                ]),
            },
            ordering: QueryOrderingContract::CanonicalRecordRefOrder,
            entities: &[0],
            relations: &[0],
            // 2 targets + 2*2 sort units + 1 group + 2 group visits + 2 copies.
            work: ordinary(11, 2, 2, 2),
        },
        Case {
            name: "entity-kind",
            scope: QueryScope::EntityKindScan {
                kind_id: KindId(1),
                partition_scope: partition.clone(),
            },
            ordering: entity_order,
            entities: &[0, 1, 2],
            relations: &[],
            work: ordinary(1, 3, 3, 1),
        },
        Case {
            name: "entity-kind-unscoped",
            scope: QueryScope::EntityKindScan {
                kind_id: KindId(1),
                partition_scope: None,
            },
            ordering: entity_order,
            entities: &[0, 1, 2],
            relations: &[],
            // One published partition discovery and one scan packet copy.
            work: ordinary(2, 3, 3, 1),
        },
        Case {
            name: "relation-kind",
            scope: QueryScope::RelationKindScan {
                kind_id: KindId(2),
                partition_scope: partition.clone(),
            },
            ordering: relation_order,
            entities: &[],
            relations: &[0, 1],
            work: ordinary(1, 2, 2, 1),
        },
        Case {
            name: "entity-equals-failing-rows",
            scope: QueryScope::EntityFieldEquals {
                field_locator: world.name.clone(),
                value: string_aspect_value("a"),
                partition_scope: partition.clone(),
            },
            ordering: entity_order,
            entities: &[0],
            relations: &[],
            work: ordinary(1, 3, 1, 1),
        },
        Case {
            name: "entity-equals-empty",
            scope: QueryScope::EntityFieldEquals {
                field_locator: world.name.clone(),
                value: string_aspect_value("absent"),
                partition_scope: partition.clone(),
            },
            ordering: entity_order,
            entities: &[],
            relations: &[],
            work: ordinary(1, 3, 0, 1),
        },
        Case {
            name: "entity-any-of",
            scope: QueryScope::EntityFieldAnyOf {
                field_locator: world.name.clone(),
                values: Arc::from([string_aspect_value("c"), string_aspect_value("a")]),
                partition_scope: partition.clone(),
            },
            ordering: entity_order,
            entities: &[0, 2],
            relations: &[],
            // 2 keys + 2*2 sort units + 1 partition + 2 cloned keys.
            work: ordinary(9, 3, 2, 1),
        },
        Case {
            name: "relation-equals-failing-row",
            scope: QueryScope::RelationFieldEquals {
                field_locator: world.label.clone(),
                value: string_aspect_value("bc"),
                partition_scope: partition.clone(),
            },
            ordering: relation_order,
            entities: &[],
            relations: &[1],
            work: ordinary(1, 2, 1, 1),
        },
        Case {
            name: "relation-any-of",
            scope: QueryScope::RelationFieldAnyOf {
                field_locator: world.label.clone(),
                values: Arc::from([string_aspect_value("bc"), string_aspect_value("ab")]),
                partition_scope: partition.clone(),
            },
            ordering: relation_order,
            entities: &[],
            relations: &[0, 1],
            work: ordinary(9, 2, 2, 1),
        },
        Case {
            name: "entity-aspect",
            scope: QueryScope::AspectFilteredEntities {
                kind_id: Some(KindId(1)),
                aspect_filter: ProjectionAspectFilter::whole_aspects(
                    ProjectionAspectFilterMode::All,
                    [aspect_key("name")],
                ),
                partition_scope: partition.clone(),
            },
            ordering: entity_order,
            entities: &[0, 1, 2],
            relations: &[],
            work: ordinary(1, 3, 3, 1),
        },
        Case {
            name: "relation-aspect",
            scope: QueryScope::AspectFilteredRelations {
                kind_id: Some(KindId(2)),
                aspect_filter: ProjectionAspectFilter::whole_aspects(
                    ProjectionAspectFilterMode::All,
                    [aspect_key("label")],
                ),
                partition_scope: partition,
            },
            ordering: relation_order,
            entities: &[],
            relations: &[0, 1],
            work: ordinary(1, 2, 2, 1),
        },
    ];
    for (name, scope, entities, relations, examined) in [
        (
            "outgoing",
            QueryScope::OutgoingNeighborhood {
                seeds: Arc::from([world.entities[0]]),
                relation_kind_scope: None,
            },
            &[0, 1][..],
            &[0][..],
            4,
        ),
        (
            "incoming",
            QueryScope::IncomingNeighborhood {
                seeds: Arc::from([world.entities[2]]),
                relation_kind_scope: None,
            },
            &[2, 1][..],
            &[1][..],
            4,
        ),
        (
            "connectivity",
            QueryScope::ConnectivityTraversal {
                seeds: Arc::from([world.entities[0]]),
                relation_kind_scope: None,
                max_depth: None,
            },
            &[0, 1, 2][..],
            &[0, 1][..],
            6,
        ),
        (
            "depth-zero-cut",
            QueryScope::ConnectivityTraversal {
                seeds: Arc::from([world.entities[0]]),
                relation_kind_scope: None,
                max_depth: Some(0),
            },
            &[0][..],
            &[][..],
            2,
        ),
    ] {
        cases.push(Case {
            name,
            scope,
            ordering: QueryOrderingContract::CanonicalTraversalOrder,
            entities,
            relations,
            work: Quote {
                // One seed + its one-unit sort + its packet copy.
                preparation: 3,
                examined,
                records: (entities.len() + relations.len()) as u64,
                captured_blocks: 1,
                output_buffers: 0,
                traversal: true,
                relation_records: relations.len() as u64,
            },
        });
    }
    cases
}

fn ordinary(preparation: u64, examined: u64, records: u64, output_buffers: u64) -> Quote {
    Quote {
        preparation,
        examined,
        records,
        // The captured Vec holds one QueryWorkerFragment header even with no rows.
        captured_blocks: 1,
        output_buffers,
        traversal: false,
        relation_records: 0,
    }
}
