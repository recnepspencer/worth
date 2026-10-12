use super::*;

#[test]
fn semantic_uniqueness_oracle_rejects_duplicate_without_mutating_branch() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let genesis = OracleBranch::genesis(OracleState::from_definition(&definition));
    let key = next_vessel_key(&genesis);
    let first =
        insert_vessel(&genesis, key, DIVERGENT_CALL_SIGN).expect("oracle accepts first value");
    let duplicate = insert_vessel(&first, next_vessel_key(&first), DIVERGENT_CALL_SIGN);
    assert_eq!(
        duplicate,
        Err(UniqueEntityFieldOracleError::DuplicateValue(
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
