pub(crate) use worth_query_bank_worlds::facade::external_rail::exit_proof::fault_disappear_mid_dispatch::*;

#[tokio::test]
async fn disappearance_leaves_no_ledger_record_at_all() {
    let rail = spawn_rail();
    let correlation = correlation_for("disappear-mid-dispatch");
    rail.select_fault(FaultScript::DisappearMidDispatch).await;

    let outcome = dispatch(
        rail.addr,
        attempt_for("disappear-mid-dispatch"),
        FRAME_TIMEOUT,
    )
    .await;

    assert_eq!(outcome, RailExchangeOutcome::Disconnected);

    let status = inquire_status(rail.addr, correlation, FRAME_TIMEOUT)
        .await
        .expect("a fresh connection can still ask the rail's ledger");
    assert_eq!(
        status,
        LedgerStatus::NoRecord,
        "unlike a lost response after commit, disappearance never touched the ledger"
    );
}
