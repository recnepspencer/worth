#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::read_contracts::expected_read};

#[test]
fn every_delta_has_an_independent_hand_authored_read_footprint() {
    for id in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL {
        assert_eq!(
            id.contract().read,
            expected_read(id),
            "read contract for {id:?}"
        );
    }
}
