#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainCostInputs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, crate::relational_certification::ordinary_context::profile_contracts::entity_kinds, crate::relational_certification::ordinary_context::profile_contracts::expected_entity_total, crate::relational_certification::ordinary_context::profile_contracts::expected_region, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_count, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_total, crate::relational_certification::ordinary_context::profile_contracts::profiles, crate::relational_certification::ordinary_context::profile_contracts::relation_kinds, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace};

#[test]
fn named_anchors_and_seeded_generation_are_stable_across_profiles() {
    let court = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let standard = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard()).unwrap();
    assert_ne!(
        court.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2)),
        standard.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2))
    );
    for scale in profiles() {
        let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        let repeat = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        assert_eq!(definition, repeat);
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(meridian) = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity()).unwrap()
        else {
            panic!("Meridian must remain a port");
        };
        assert_eq!(meridian.name, "Meridian");
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(southpoint) =
            definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity()).unwrap()
        else {
            panic!("Southpoint must remain a port");
        };
        assert_eq!(southpoint.name, "Southpoint");
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) =
            definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity()).unwrap()
        else {
            panic!("Medical Supplies must remain cargo");
        };
        assert_eq!(cargo.customer_code.0, "CARGO-MEDICAL-0000");
        for anchor in scale.anchors() {
            assert!(
                definition.entity(anchor.entity()).is_some(),
                "missing {anchor}"
            );
        }
    }
}
