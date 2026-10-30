pub(crate) use worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge;

#[test]
fn schema_has_all_entity_and_relation_contracts() {
    let schema = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    assert_eq!(schema.relations.len(), 10);
    for (kind, relation) in &schema.relations {
        assert_eq!(*kind, relation.kind);
    }
    assert_eq!(schema.version, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    assert_eq!(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ALL.len(), 20);
}

#[test]
fn operating_definition_has_exact_court_counts_and_named_anchors() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    assert_eq!(definition.entities.len(), 244);
    assert_eq!(definition.relations.len(), 247);
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(port) = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity()).unwrap() else {
        panic!("Meridian must be a port");
    };
    assert_eq!(port.name, "Meridian");
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(vessel) = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity()).unwrap() else {
        panic!("Aurora must be a vessel");
    };
    assert_eq!(vessel.call_sign, "AURORA");
}

#[test]
fn empty_definition_has_no_records_and_retains_schema() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert!(definition.entities.is_empty());
    assert!(definition.relations.is_empty());
    assert_eq!(definition.schema.version, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
}

#[test]
fn invalid_relation_endpoint_is_typed() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let edge = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort, 90_000),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity(),
    };
    let source = definition.entity(edge.source).unwrap().kind();
    let target = definition.entity(edge.target).unwrap().kind();
    let error = definition
        .schema
        .validate_edge(edge, source, target)
        .unwrap_err();
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint { .. }));
}

#[test]
fn duplicate_cardinality_and_route_cycle_failures_are_typed() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let duplicate = *definition.relations.values().next().unwrap();
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[duplicate, duplicate], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::DuplicateRelation(_))
    ));

    let first = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel, 50_000),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity(),
    };
    let second = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel, 50_001),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[first, second], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::CardinalityExceeded(_, _))
    ));

    let cycle = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 50_002),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[cycle], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::RouteCycle)
    ));
}

#[test]
fn complete_schema_validation_enforces_minimum_symmetry_order_and_multi_node_cycles() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    assert!(matches!(
        definition
            .schema
            .validate_complete_world(&[], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MinimumCardinality(_, _))
    ));

    let one_way = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone, 9_000),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[one_way], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MissingSymmetricReverse(_))
    ));

    let ordered = [
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall, 9_001),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
        },
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall, 9_002),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&ordered, &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrderedRouteViolation(_))
    ));

    let cycle = [
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 9_003),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
        },
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 9_004),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&cycle, &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::RouteCycle)
    ));
}

#[test]
fn complete_routes_require_unique_ownership_and_each_predecessor_link() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let first_port = definition
        .relations
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 0))
        .unwrap()
        .target;
    let second_port = definition
        .relations
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1))
        .unwrap()
        .target;
    let (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(first), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(second)) = (
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
    missing_link.retain(|key, _| key.kind != worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes);
    assert!(matches!(
        definition.schema.validate_complete_world(
            &missing_link.values().copied().collect::<Vec<_>>(),
            &definition.entities,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MissingRouteLink { .. })
    ));

    let mut duplicate = definition.relations.values().copied().collect::<Vec<_>>();
    duplicate.push(RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall, 90_000),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 1),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
    });
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&duplicate, &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::DuplicateVoyageCall(_))
    ));

    let orphan = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 90_001),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity(),
    };
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&[orphan], &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(_))
    ));

    let cross_voyage = [
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall, 90_002),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
        },
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall, 90_003),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 1),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 3),
        },
        RelationEdge {
            key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 90_004),
            source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(),
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 3),
        },
    ];
    assert!(matches!(
        definition
            .schema
            .validate_relation_sequence(&cross_voyage, &definition.entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(_))
    ));
}

#[test]
fn complete_routes_reject_sequence_gaps_and_duplicate_endpoint_links() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let mut entities = definition.entities.clone();
    let call_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(mut call) = entities.remove(&call_key).unwrap() else {
        panic!("Southpoint must be a port call");
    };
    call.sequence = 3;
    entities.insert(call_key, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(call));
    let relations = definition.relations.values().copied().collect::<Vec<_>>();
    assert!(matches!(
        definition
            .schema
            .validate_complete_world(&relations, &entities),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrderedRouteViolation(_))
    ));

    let existing = relations
        .iter()
        .find(|edge| edge.key.kind == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes)
        .copied()
        .expect("court route must have a predecessor link");
    let mut duplicate_link = relations;
    duplicate_link.push(RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 90_005),
        source: existing.source,
        target: existing.target,
    });
    assert_eq!(
        definition
            .schema
            .validate_complete_world(&duplicate_link, &definition.entities,)
            .unwrap_err(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes, 90_005,))
    );
}

#[test]
fn hazard_meaning_cannot_cross_the_schema_boundary() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let cargo_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity();
    let mut entities = definition.entities.clone();
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(mut cargo) = entities.remove(&cargo_key).unwrap() else {
        panic!("medical supplies must be cargo");
    };
    cargo.hazard = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2;
    entities.insert(cargo_key, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo));
    assert!(matches!(
        definition.schema.validate_complete_world(
            &definition.relations.values().copied().collect::<Vec<_>>(),
            &entities,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::HazardMeaningViolation { .. })
    ));
}
