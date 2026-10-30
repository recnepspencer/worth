#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare};

#[test]
fn retention_pressure_can_record_entity_and_relation_deletion_markers() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::retention_pressure(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let entity = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MachineParts.entity();
    let relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage, 0);
    let state = baseline
        .branch
        .state
        .remove_entity(entity)
        .remove_relation(relation);
    assert_eq!(
        state.absence_marker(entity),
        Some((worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind::Entity, entity))
    );
    assert_eq!(
        state.relation_absence_marker(relation),
        Some((worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind::Relation, relation))
    );
    let branch = OracleBranch {
        state,
        ancestry: baseline.branch.ancestry,
    };
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&branch);
    assert_eq!(
        compare(
            &expected,
            &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected)
        ),
        Ok(())
    );
}
