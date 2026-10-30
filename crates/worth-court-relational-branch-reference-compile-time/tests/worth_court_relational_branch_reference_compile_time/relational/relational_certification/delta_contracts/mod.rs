#[allow(unused_imports)]
use crate::relational_certification::ordinary_context::delta_contracts::*;

#[test]
fn rewiring_preserves_relation_identity_and_hazard_v2_is_schema_typed() {
    let rewire = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall.contract();
    let call_relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    assert!(rewire.footprint.relations.contains(&call_relation));
    assert!(rewire
        .postconditions
        .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationTargetChanges(call_relation)));
    assert!(rewire
        .postconditions
        .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationSourceUnchanged(call_relation)));
    assert!(rewire
        .identity_basis
        .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaIdentityBasis::Relation(call_relation)));

    let hazard = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2.contract();
    assert_eq!(hazard.schema_meaning, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2);
    assert!(hazard
        .postconditions
        .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::SchemaChanges(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2)));
}
