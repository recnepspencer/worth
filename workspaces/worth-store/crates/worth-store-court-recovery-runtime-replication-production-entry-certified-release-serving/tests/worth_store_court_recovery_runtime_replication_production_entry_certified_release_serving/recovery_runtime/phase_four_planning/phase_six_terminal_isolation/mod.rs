use {worth_store_recovery_runtime::PhysicalRecoveryOutcome};

#[test]
fn recovered_session_evidence_is_local_despite_an_unrelated_nonterminal_drop() {
    let dropped_root = crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-unrelated-drop");
    let dropped = crate::phase_four_planning::selected_ordinary_recovery(dropped_root.path());
    let drop_thread = std::thread::spawn(move || drop(dropped));

    let recovered_root = crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-local-terminal");
    let admitted = worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits(
        recovered_root.path(),
        crate::phase_four_planning::ordinary_recovery_limits(4_096),
    );
    let expected_session = admitted.session_identity();
    let reopened = admitted
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap()
        .reopen()
        .unwrap();
    drop_thread.join().expect("unrelated drop thread");

    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("the consumed local session must retain its recovered terminal")
    };
    assert_eq!(
        handoff.publication_expectation().recovered_root(),
        handoff.core().root()
    );
    assert_eq!(handoff.recovered_session_identity(), expected_session);
}
