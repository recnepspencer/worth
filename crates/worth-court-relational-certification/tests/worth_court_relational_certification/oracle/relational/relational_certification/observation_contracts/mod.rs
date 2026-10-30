#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::canonical_bytes, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::digest, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[test]
fn every_delta_has_a_distinct_expected_semantic_observation() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let mut digests = std::collections::BTreeSet::new();
    for delta in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL {
        let branch = baseline
            .branch
            .fork(delta.branch(), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .unwrap();
        let next = apply(&branch, delta).unwrap();
        let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&next);
        assert!(digests.insert(digest(&expected)));
    }
}

#[test]
fn expected_observation_is_distinct_from_oracle_state() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    assert_eq!(expected.schema_version(), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    assert_eq!(expected.entities, baseline.branch.state.entities);
    assert_eq!(expected.relations, baseline.branch.state.relations);
    assert_eq!(
        baseline
            .branch
            .state
            .absence_marker(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()),
        None
    );
    let relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    assert_eq!(
        baseline.branch.state.relation_absence_marker(relation),
        None
    );
}

#[test]
fn hand_authored_delta_vectors_pin_changed_and_unchanged_facts() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let storm = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = storm
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("Aurora voyage vector must remain a voyage");
    };
    assert_eq!(voyage.status, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Rerouted);
    assert_eq!(voyage.arrival.0, 230);
    assert_eq!(
        storm.entities.get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity()),
        baseline
            .branch
            .state
            .entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity())
    );

    let maintenance = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(berth) = maintenance.entities.get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity()).unwrap()
    else {
        panic!("Atlas vector must remain a berth");
    };
    assert_eq!(berth.posture, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Maintenance);
    assert_eq!(
        maintenance
            .relations
            .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0))
            .unwrap()
            .target,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Beacon.entity()
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = maintenance
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("maintenance vector must remain a voyage");
    };
    assert_eq!(voyage.status, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed);
    assert_eq!(voyage.arrival.0, 260);

    let expanded = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(terminal) = expanded
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, 1))
        .unwrap()
    else {
        panic!("Southpoint expansion vector must remain a terminal");
    };
    assert_eq!(terminal.capacity.0, 11_100);
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(berth) = expanded
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity())
        .unwrap()
    else {
        panic!("Southpoint expansion vector must remain a berth");
    };
    assert_eq!(berth.capacity.0, 2_070);

    let hold = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) = hold
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity())
        .unwrap()
    else {
        panic!("medical vector must remain cargo");
    };
    assert_eq!(cargo.booking, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Held);

    let competing = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = competing
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("competing arrival vector must remain a voyage");
    };
    assert_eq!(voyage.status, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed);
    assert_eq!(voyage.arrival.0, 250);

    let rewired = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
        )
        .unwrap(),
    );
    assert_eq!(
        rewired
            .relations
            .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1))
            .unwrap()
            .target,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3)
    );

    let hazard = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
        )
        .unwrap(),
    );
    assert_eq!(hazard.schema.version, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2);
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) = hazard
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity())
        .unwrap()
    else {
        panic!("hazard vector must remain cargo");
    };
    assert_eq!(cargo.hazard, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2);

    let inspection = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .unwrap(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora,
        )
        .unwrap(),
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(value) = inspection
        .entities
        .get(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity())
        .unwrap()
    else {
        panic!("inspection vector must remain an inspection");
    };
    assert_eq!(value.result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Flagged);
}
