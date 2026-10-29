#[allow(unused_imports)] use crate::relational_certification::ordinary_context::read_contracts::*;
pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::read_footprint::DeltaReadFootprint, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[test]
fn read_footprint_mutations_are_detected_on_each_history_and_entity_axis() {
    let contract = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora.contract();
    let mut missing_entity = expected_read(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora);
    missing_entity
        .entities
        .remove(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2));
    assert_ne!(missing_entity, contract.read);

    let mut missing_relation = expected_read(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora);
    missing_relation
        .relations
        .remove(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1));
    assert_ne!(missing_relation, contract.read);

    let mut missing_field = expected_read(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora);
    missing_field.fields.remove(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision,
    ));
    assert_ne!(missing_field, contract.read);

    let mut wrong_history = expected_read(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora);
    wrong_history.history = Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival);
    assert_ne!(wrong_history, contract.read);
}
