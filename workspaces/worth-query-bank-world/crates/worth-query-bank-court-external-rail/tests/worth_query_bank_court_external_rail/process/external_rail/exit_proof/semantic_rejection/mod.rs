pub(crate) use worth_query_bank_testkit_external_rail::facade::exit_proof::semantic_rejection::*;

#[tokio::test]
async fn hostile_protocol_matrix_rejects_before_ledger_and_healthy_twin_completes() {
    let rail = spawn_rail();
    for (index, hostile) in hostile_cases().into_iter().enumerate() {
        assert_hostile_rejection(rail.addr, hostile, index as u8).await;
    }
    assert_healthy_twin(rail.addr).await;
}
