pub(crate) use {crate::relational_certification::ordinary_context::invariant_global_uniqueness::DIVERGENT_CALL_SIGN, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::insert_vessel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::next_vessel_key, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::vessel_call_signs as oracle_vessel_call_signs};

#[test]
fn semantic_uniqueness_oracle_rejects_duplicate_without_mutating_branch() {
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let genesis = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch::genesis(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState::from_definition(&definition));
    let key = next_vessel_key(&genesis);
    let first =
        insert_vessel(&genesis, key, DIVERGENT_CALL_SIGN).expect("oracle accepts first value");
    let duplicate = insert_vessel(&first, next_vessel_key(&first), DIVERGENT_CALL_SIGN);
    assert_eq!(
        duplicate,
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError::DuplicateValue(
            DIVERGENT_CALL_SIGN.to_owned()
        ))
    );
    assert_eq!(
        oracle_vessel_call_signs(&first),
        oracle_vessel_call_signs(&genesis)
            .into_iter()
            .chain([DIVERGENT_CALL_SIGN.to_owned()])
            .collect::<Vec<_>>()
    );
}
