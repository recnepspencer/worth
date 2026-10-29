pub(crate) use worth_query_bank_worlds::facade::external_rail::exit_proof::fault_duplicate_acknowledgement::*;

#[tokio::test]
async fn duplicate_acknowledgement_is_distinct_from_completion() {
    let rail = spawn_rail();
    let correlation = correlation_for("duplicate-acknowledgement");
    rail.select_fault(FaultScript::DuplicateAcknowledgement)
        .await;

    let outcome = dispatch(
        rail.addr,
        attempt_for("duplicate-acknowledgement"),
        FRAME_TIMEOUT,
    )
    .await;

    assert_eq!(outcome, RailExchangeOutcome::DuplicateAcknowledgement);

    let status = inquire_status(rail.addr, correlation, FRAME_TIMEOUT)
        .await
        .expect("a fresh connection can still ask the rail's ledger");
    assert_eq!(
        status,
        LedgerStatus::Acknowledged,
        "a duplicate acknowledgement is still only an acknowledgement, never a completion"
    );
}
