use crate::relational_certification::world::supply_chain::*;

#[test]
fn production_snapshot_schema_ignores_program_descriptor_mutation() {
    let host_request = crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let program = CompiledSupplyChainProgram::compile(definition).unwrap();
    let world = compile_supply_chain_baseline(execution, program).unwrap();
    let mut altered_program = world.program.clone();
    altered_program.definition_mut_for_test().schema =
        SupplyChainSchema::canonical(SchemaVersion::V2);

    let observed = observe_supply_chain_snapshot(
        execution,
        &altered_program,
        &world.handles,
        &world.runtime,
        &world.handles.snapshot,
    )
    .expect("the owner-selected production snapshot remains observable");

    assert_eq!(observed.schema, SchemaVersion::V1);
    assert_ne!(
        observed.schema,
        altered_program.definition().schema.version,
        "schema observation must not copy the caller's program descriptor"
    );
}

#[test]
fn expected_observation_is_distinct_from_oracle_state() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    assert_eq!(expected.schema_version(), SchemaVersion::V1);
    assert_eq!(expected.entities, baseline.branch.state.entities);
    assert_eq!(expected.relations, baseline.branch.state.relations);
    assert_eq!(
        baseline
            .branch
            .state
            .absence_marker(Anchor::AuroraEastbound.entity()),
        None
    );
    let relation = RelationKey::new(RelationKind::CallAtPort, 1);
    assert_eq!(
        baseline.branch.state.relation_absence_marker(relation),
        None
    );
}

#[test]
fn canonical_observation_bytes_are_versioned_and_ordered() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let bytes = canonical_bytes(&expected);
    assert!(bytes.starts_with(b"supply-chain-expected-v1\0"));
    assert_eq!(bytes, canonical_bytes(&expected));
    assert_eq!(
        digest(&expected),
        [
            0x42, 0x7b, 0xf1, 0x58, 0x55, 0xb8, 0x11, 0xf5, 0x64, 0x3c, 0xf1, 0x0a, 0x82, 0x72,
            0x2d, 0xec, 0x76, 0x73, 0xc5, 0x5f, 0x53, 0x88, 0x99, 0x29, 0x53, 0x18, 0x55, 0x3e,
            0x05, 0xfe, 0x50, 0xb2,
        ]
    );
}

#[test]
fn canonical_observation_is_invariant_to_relation_insertion_permutation() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut permuted = expected.clone();
    let relations: Vec<_> = expected.relations.values().copied().rev().collect();
    permuted.relations.clear();
    for edge in relations {
        permuted.relations.insert(edge.key, edge);
    }
    assert_eq!(canonical_bytes(&expected), canonical_bytes(&permuted));
    assert_eq!(digest(&expected), digest(&permuted));
}

#[test]
fn every_delta_has_a_distinct_expected_semantic_observation() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let mut digests = std::collections::BTreeSet::new();
    for delta in DeltaId::ALL {
        let branch = baseline
            .branch
            .fork(delta.branch(), BranchLabel::Operating)
            .unwrap();
        let next = apply(&branch, delta).unwrap();
        let expected = ExpectedSupplyChainObservation::from_branch(&next);
        assert!(digests.insert(digest(&expected)));
    }
}

#[test]
fn hand_authored_delta_vectors_pin_changed_and_unchanged_facts() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let storm = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::Storm, BranchLabel::Operating)
                .unwrap(),
            DeltaId::StormRerouteAurora,
        )
        .unwrap(),
    );
    let EntityRecord::Voyage(voyage) = storm
        .entities
        .get(&Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("Aurora voyage vector must remain a voyage");
    };
    assert_eq!(voyage.status, VoyageStatus::Rerouted);
    assert_eq!(voyage.arrival.0, 230);
    assert_eq!(
        storm.entities.get(&Anchor::MedicalSupplies.entity()),
        baseline
            .branch
            .state
            .entity(Anchor::MedicalSupplies.entity())
    );

    let maintenance = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::Maintenance, BranchLabel::Operating)
                .unwrap(),
            DeltaId::MaintainAtlasBerth,
        )
        .unwrap(),
    );
    let EntityRecord::Berth(berth) = maintenance.entities.get(&Anchor::Atlas.entity()).unwrap()
    else {
        panic!("Atlas vector must remain a berth");
    };
    assert_eq!(berth.posture, OperatingPosture::Maintenance);
    assert_eq!(
        maintenance
            .relations
            .get(&RelationKey::new(RelationKind::VesselAssignedToBerth, 0))
            .unwrap()
            .target,
        Anchor::Beacon.entity()
    );
    let EntityRecord::Voyage(voyage) = maintenance
        .entities
        .get(&Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("maintenance vector must remain a voyage");
    };
    assert_eq!(voyage.status, VoyageStatus::Delayed);
    assert_eq!(voyage.arrival.0, 260);

    let expanded = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::SouthpointExpansion, BranchLabel::Operating)
                .unwrap(),
            DeltaId::ExpandSouthpointCapacity,
        )
        .unwrap(),
    );
    let EntityRecord::Terminal(terminal) = expanded
        .entities
        .get(&EntityKey::new(EntityKind::Terminal, 1))
        .unwrap()
    else {
        panic!("Southpoint expansion vector must remain a terminal");
    };
    assert_eq!(terminal.capacity.0, 11_100);
    let EntityRecord::Berth(berth) = expanded
        .entities
        .get(&Anchor::SouthpointBerth.entity())
        .unwrap()
    else {
        panic!("Southpoint expansion vector must remain a berth");
    };
    assert_eq!(berth.capacity.0, 2_070);

    let hold = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::MedicalHold, BranchLabel::Operating)
                .unwrap(),
            DeltaId::HoldMedicalCargo,
        )
        .unwrap(),
    );
    let EntityRecord::CargoLot(cargo) = hold
        .entities
        .get(&Anchor::MedicalSupplies.entity())
        .unwrap()
    else {
        panic!("medical vector must remain cargo");
    };
    assert_eq!(cargo.booking, BookingStatus::Held);

    let competing = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::CompetingArrival, BranchLabel::Operating)
                .unwrap(),
            DeltaId::CompetingAuroraArrival,
        )
        .unwrap(),
    );
    let EntityRecord::Voyage(voyage) = competing
        .entities
        .get(&Anchor::AuroraEastbound.entity())
        .unwrap()
    else {
        panic!("competing arrival vector must remain a voyage");
    };
    assert_eq!(voyage.status, VoyageStatus::Delayed);
    assert_eq!(voyage.arrival.0, 250);

    let rewired = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::Rewire, BranchLabel::Operating)
                .unwrap(),
            DeltaId::RewireAuroraPortCall,
        )
        .unwrap(),
    );
    assert_eq!(
        rewired
            .relations
            .get(&RelationKey::new(RelationKind::CallAtPort, 1))
            .unwrap()
            .target,
        EntityKey::new(EntityKind::Port, 3)
    );

    let hazard = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::HazardV2, BranchLabel::Operating)
                .unwrap(),
            DeltaId::AdoptHazardClassificationV2,
        )
        .unwrap(),
    );
    assert_eq!(hazard.schema.version, SchemaVersion::V2);
    let EntityRecord::CargoLot(cargo) = hazard
        .entities
        .get(&Anchor::MedicalSupplies.entity())
        .unwrap()
    else {
        panic!("hazard vector must remain cargo");
    };
    assert_eq!(cargo.hazard, HazardClass::HazardousV2);

    let inspection = ExpectedSupplyChainObservation::from_branch(
        &apply(
            &baseline
                .branch
                .fork(BranchLabel::Inspection, BranchLabel::Operating)
                .unwrap(),
            DeltaId::RetireAtlasWhileInspectingAurora,
        )
        .unwrap(),
    );
    let EntityRecord::Inspection(value) = inspection
        .entities
        .get(&Anchor::AuroraArrival.entity())
        .unwrap()
    else {
        panic!("inspection vector must remain an inspection");
    };
    assert_eq!(value.result, InspectionResult::Flagged);
}
