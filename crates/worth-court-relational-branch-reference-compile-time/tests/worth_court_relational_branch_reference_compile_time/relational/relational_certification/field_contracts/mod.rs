pub(crate) use crate::relational_certification::ordinary_context::field_values::field_value;

#[test]
fn field_vocabulary_is_exhaustive_and_not_aliased() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let records = [
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(), 4),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MeridianContainer.entity(), 3),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity(), 4),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity(), 4),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(), 4),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity(), 2),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity(), 4),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity(), 2),
    ];
    for (key, expected) in records {
        let record = definition.entity(key).unwrap();
        let supported = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ALL
            .into_iter()
            .filter(|field| field_value(record, *field).is_some())
            .count();
        assert_eq!(
            supported, expected,
            "incomplete field vocabulary for {key:?}"
        );
    }
    let vessel = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora.entity()).unwrap();
    assert!(field_value(vessel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Name).is_none());
    let inspection = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity()).unwrap();
    assert!(field_value(inspection, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute).is_none());
}
