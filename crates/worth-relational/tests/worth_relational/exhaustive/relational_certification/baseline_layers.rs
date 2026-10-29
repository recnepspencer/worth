#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::baseline_layers::*;

#[test]
fn retention_pressure_can_record_entity_and_relation_deletion_markers() {
    let baseline = SupplyChainBaseline::retention_pressure(SupplyChainScale::court());
    let entity = Anchor::MachineParts.entity();
    let relation = RelationKey::new(RelationKind::CargoBookedOnVoyage, 0);
    let state = baseline
        .branch
        .state
        .remove_entity(entity)
        .remove_relation(relation);
    assert_eq!(
        state.absence_marker(entity),
        Some((AbsenceKind::Entity, entity))
    );
    assert_eq!(
        state.relation_absence_marker(relation),
        Some((AbsenceKind::Relation, relation))
    );
    let branch = OracleBranch {
        state,
        ancestry: baseline.branch.ancestry,
    };
    let expected = ExpectedSupplyChainObservation::from_branch(&branch);
    assert_eq!(
        compare(
            &expected,
            &ObservedSupplyChainState::from_expected(&expected)
        ),
        Ok(())
    );
}
