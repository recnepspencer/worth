pub(crate) use worth_query_bank_testkit_external_rail::facade::exit_proof::fault_lost_response::*;

#[tokio::test]
async fn commit_then_lose_response_never_reports_completed_to_the_caller() {
    let rail = spawn_rail();
    let correlation = correlation_for("lost-response");
    rail.select_fault(FaultScript::CommitThenLoseResponse).await;

    let outcome = dispatch(rail.addr, attempt_for("lost-response"), FRAME_TIMEOUT).await;

    assert_eq!(outcome, RailExchangeOutcome::Disconnected);

    let status = inquire_status(rail.addr, correlation, FRAME_TIMEOUT)
        .await
        .expect("a fresh connection can still ask the rail's ledger");
    assert_eq!(
        status,
        LedgerStatus::Completed,
        "the rail's own truth shows the effect committed, even though the caller lost the reply"
    );
}
