//! A stale prepared correction keeps terminal custody instead of retrying as a reversal.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted,
};

#[test]
fn stale_prepared_correction_is_terminal_and_cannot_retry_as_an_ordinary_reversal() {
    let world = fixture::World::new(9);
    let (source, _) = world.send(30, "stale-original");
    let scope = request_scope();
    let request = world.native.runtime.request(&world.teller, &scope);
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source.coordinates(),
            "stale-inverse",
            limits(),
        )
        .unwrap()
    else {
        panic!("prepare at exact current target");
    };
    world.send(7, "stale-target-movement");
    let before = world.activity(world.source);
    let destination_before = world.activity(world.destination);
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = prepared.publish().unwrap() else {
        panic!("stale correction retains terminal custody without an ordinary retry");
    };
    assert!(
        matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause))) if cause.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale)
    );
    assert_eq!(world.balances(), (63, 37));
    assert_eq!(world.activity(world.source), before);
    assert_eq!(world.activity(world.destination), destination_before);
    assert!(terminal.close(&request).is_ok());
}
