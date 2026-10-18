use crate::relational_certification::field_values::field_value;
use crate::relational_certification::world::supply_chain::*;

#[test]
fn field_vocabulary_is_exhaustive_and_not_aliased() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let records = [
        (Anchor::Meridian.entity(), 4),
        (Anchor::MeridianContainer.entity(), 3),
        (Anchor::Atlas.entity(), 4),
        (Anchor::Aurora.entity(), 4),
        (Anchor::AuroraEastbound.entity(), 4),
        (Anchor::AuroraMeridian.entity(), 2),
        (Anchor::MedicalSupplies.entity(), 4),
        (Anchor::AuroraArrival.entity(), 2),
    ];
    for (key, expected) in records {
        let record = definition.entity(key).unwrap();
        let supported = FieldKey::ALL
            .into_iter()
            .filter(|field| field_value(record, *field).is_some())
            .count();
        assert_eq!(
            supported, expected,
            "incomplete field vocabulary for {key:?}"
        );
    }
    let vessel = definition.entity(Anchor::Aurora.entity()).unwrap();
    assert!(field_value(vessel, FieldKey::Name).is_none());
    let inspection = definition.entity(Anchor::AuroraArrival.entity()).unwrap();
    assert!(field_value(inspection, FieldKey::ArrivalMinute).is_none());
}
