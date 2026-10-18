use crate::relational_certification::world::supply_chain::*;

#[test]
fn schema_has_all_entity_and_relation_contracts() {
    let schema = SupplyChainSchema::canonical(SchemaVersion::V1);
    assert_eq!(schema.relations.len(), 10);
    for (kind, relation) in &schema.relations {
        assert_eq!(*kind, relation.kind);
    }
    assert_eq!(schema.version, SchemaVersion::V1);
    assert_eq!(FieldKey::ALL.len(), 20);
}

#[test]
fn operating_definition_has_exact_court_counts_and_named_anchors() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    assert_eq!(definition.entities.len(), 244);
    assert_eq!(definition.relations.len(), 247);
    let EntityRecord::Port(port) = definition.entity(Anchor::Meridian.entity()).unwrap() else {
        panic!("Meridian must be a port");
    };
    assert_eq!(port.name, "Meridian");
    let EntityRecord::Vessel(vessel) = definition.entity(Anchor::Aurora.entity()).unwrap() else {
        panic!("Aurora must be a vessel");
    };
    assert_eq!(vessel.call_sign, "AURORA");
}

#[test]
fn empty_definition_has_no_records_and_retains_schema() {
    let definition = SupplyChainWorldDefinition::empty(SupplyChainScale::court());
    assert!(definition.entities.is_empty());
    assert!(definition.relations.is_empty());
    assert_eq!(definition.schema.version, SchemaVersion::V1);
}

#[test]
fn invalid_relation_endpoint_is_typed() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let edge = RelationEdge {
        key: RelationKey::new(RelationKind::TerminalAtPort, 90_000),
        source: Anchor::Meridian.entity(),
        target: Anchor::Southpoint.entity(),
    };
    let source = definition.entity(edge.source).unwrap().kind();
    let target = definition.entity(edge.target).unwrap().kind();
    let error = definition
        .schema
        .validate_edge(edge, source, target)
        .unwrap_err();
    assert!(matches!(error, SchemaError::InvalidEndpoint { .. }));
}

#[test]
fn duplicate_cardinality_and_route_cycle_failures_are_typed() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let duplicate = *definition.relations.values().next().unwrap();
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[duplicate, duplicate], &definition.entities),
        Err(SchemaError::DuplicateRelation(_))
    ));

    let first = RelationEdge {
        key: RelationKey::new(RelationKind::VoyageUsesVessel, 50_000),
        source: Anchor::AuroraEastbound.entity(),
        target: Anchor::Aurora.entity(),
    };
    let second = RelationEdge {
        key: RelationKey::new(RelationKind::VoyageUsesVessel, 50_001),
        source: Anchor::AuroraEastbound.entity(),
        target: Anchor::Aurora.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[first, second], &definition.entities),
        Err(SchemaError::CardinalityExceeded(_, _))
    ));

    let cycle = RelationEdge {
        key: RelationKey::new(RelationKind::CallPrecedes, 50_002),
        source: Anchor::AuroraSouthpoint.entity(),
        target: Anchor::AuroraSouthpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[cycle], &definition.entities),
        Err(SchemaError::RouteCycle)
    ));
}

#[test]
fn complete_schema_validation_enforces_minimum_symmetry_order_and_multi_node_cycles() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    assert!(matches!(
        definition
            .schema
            .validate_complete_world(&[], &definition.entities),
        Err(SchemaError::MinimumCardinality(_, _))
    ));

    let one_way = RelationEdge {
        key: RelationKey::new(RelationKind::SharesPilotageZone, 9_000),
        source: Anchor::Meridian.entity(),
        target: Anchor::Southpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[one_way], &definition.entities),
        Err(SchemaError::MissingSymmetricReverse(_))
    ));

    let ordered = [
        RelationEdge {
            key: RelationKey::new(RelationKind::VoyageHasCall, 9_001),
            source: Anchor::AuroraEastbound.entity(),
            target: Anchor::AuroraSouthpoint.entity(),
        },
        RelationEdge {
            key: RelationKey::new(RelationKind::VoyageHasCall, 9_002),
            source: Anchor::AuroraEastbound.entity(),
            target: Anchor::AuroraMeridian.entity(),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&ordered, &definition.entities),
        Err(SchemaError::OrderedRouteViolation(_))
    ));

    let cycle = [
        RelationEdge {
            key: RelationKey::new(RelationKind::CallPrecedes, 9_003),
            source: Anchor::AuroraMeridian.entity(),
            target: Anchor::AuroraSouthpoint.entity(),
        },
        RelationEdge {
            key: RelationKey::new(RelationKind::CallPrecedes, 9_004),
            source: Anchor::AuroraSouthpoint.entity(),
            target: Anchor::AuroraMeridian.entity(),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&cycle, &definition.entities),
        Err(SchemaError::RouteCycle)
    ));
}

#[test]
fn complete_routes_require_unique_ownership_and_each_predecessor_link() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let first_port = definition
        .relations
        .get(&RelationKey::new(RelationKind::CallAtPort, 0))
        .unwrap()
        .target;
    let second_port = definition
        .relations
        .get(&RelationKey::new(RelationKind::CallAtPort, 1))
        .unwrap()
        .target;
    let (EntityRecord::Port(first), EntityRecord::Port(second)) = (
        definition.entity(first_port).unwrap(),
        definition.entity(second_port).unwrap(),
    ) else {
        panic!("route calls must target ports");
    };
    assert_ne!(first.region, second.region);
    assert!(definition
        .schema
        .validate_complete_world(
            &definition.relations.values().copied().collect::<Vec<_>>(),
            &definition.entities,
        )
        .is_ok());

    let mut missing_link = definition.relations.clone();
    missing_link.retain(|key, _| key.kind != RelationKind::CallPrecedes);
    assert!(matches!(
        definition.schema.validate_complete_world(
            &missing_link.values().copied().collect::<Vec<_>>(),
            &definition.entities,
        ),
        Err(SchemaError::MissingRouteLink { .. })
    ));

    let mut duplicate = definition.relations.values().copied().collect::<Vec<_>>();
    duplicate.push(RelationEdge {
        key: RelationKey::new(RelationKind::VoyageHasCall, 90_000),
        source: EntityKey::new(EntityKind::Voyage, 1),
        target: Anchor::AuroraMeridian.entity(),
    });
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&duplicate, &definition.entities),
        Err(SchemaError::DuplicateVoyageCall(_))
    ));

    let orphan = RelationEdge {
        key: RelationKey::new(RelationKind::CallPrecedes, 90_001),
        source: Anchor::AuroraMeridian.entity(),
        target: Anchor::AuroraSouthpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[orphan], &definition.entities),
        Err(SchemaError::OrphanRouteLink(_))
    ));

    let cross_voyage = [
        RelationEdge {
            key: RelationKey::new(RelationKind::VoyageHasCall, 90_002),
            source: EntityKey::new(EntityKind::Voyage, 0),
            target: Anchor::AuroraMeridian.entity(),
        },
        RelationEdge {
            key: RelationKey::new(RelationKind::VoyageHasCall, 90_003),
            source: EntityKey::new(EntityKind::Voyage, 1),
            target: EntityKey::new(EntityKind::PortCall, 3),
        },
        RelationEdge {
            key: RelationKey::new(RelationKind::CallPrecedes, 90_004),
            source: Anchor::AuroraMeridian.entity(),
            target: EntityKey::new(EntityKind::PortCall, 3),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&cross_voyage, &definition.entities),
        Err(SchemaError::OrphanRouteLink(_))
    ));
}

#[test]
fn complete_routes_reject_sequence_gaps_and_duplicate_endpoint_links() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let mut entities = definition.entities.clone();
    let call_key = Anchor::AuroraSouthpoint.entity();
    let EntityRecord::PortCall(mut call) = entities.remove(&call_key).unwrap() else {
        panic!("Southpoint must be a port call");
    };
    call.sequence = 3;
    entities.insert(call_key, EntityRecord::PortCall(call));
    let relations = definition.relations.values().copied().collect::<Vec<_>>();
    assert!(matches!(
        definition
            .schema
            .validate_complete_world(&relations, &entities),
        Err(SchemaError::OrderedRouteViolation(_))
    ));

    let existing = relations
        .iter()
        .find(|edge| edge.key.kind == RelationKind::CallPrecedes)
        .copied()
        .expect("court route must have a predecessor link");
    let mut duplicate_link = relations;
    duplicate_link.push(RelationEdge {
        key: RelationKey::new(RelationKind::CallPrecedes, 90_005),
        source: existing.source,
        target: existing.target,
    });
    assert_eq!(
        definition
            .schema
            .validate_complete_world(&duplicate_link, &definition.entities,)
            .unwrap_err(),
        SchemaError::OrphanRouteLink(RelationKey::new(RelationKind::CallPrecedes, 90_005,))
    );
}

#[test]
fn hazard_meaning_cannot_cross_the_schema_boundary() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let cargo_key = Anchor::MedicalSupplies.entity();
    let mut entities = definition.entities.clone();
    let EntityRecord::CargoLot(mut cargo) = entities.remove(&cargo_key).unwrap() else {
        panic!("medical supplies must be cargo");
    };
    cargo.hazard = HazardClass::HazardousV2;
    entities.insert(cargo_key, EntityRecord::CargoLot(cargo));
    assert!(matches!(
        definition.schema.validate_complete_world(
            &definition.relations.values().copied().collect::<Vec<_>>(),
            &entities,
        ),
        Err(SchemaError::HazardMeaningViolation { .. })
    ));
}
