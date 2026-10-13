use super::*;

pub(super) struct Case {
    pub name: &'static str,
    pub scope: QueryScope,
    pub ordering: QueryOrderingContract,
    pub entities: &'static [usize],
    pub relations: &'static [usize],
}

// A one-partition fixture: three named vertices and the chain a -> b -> c.
// "a" fails the predicate on two rows; "absent" fails on all three.
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
        },
    ];
    for (name, scope, entities, relations, _examined) in [
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
        });
    }
    cases
}
