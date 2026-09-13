use super::*;
#[test]
fn pending_relational_settlement_never_calls_signal_or_republishes_product() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let candidate = court
        .records
        .candidate(root.basis().relational_basis(), "grain", "5");
    court.records.runtime.fail_next_durable_append_for_test();
    let token = RuntimeWorldCancellationSource::new().token();
    let port = court.world.publication_port();
    let prepared = port
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            &token,
            None,
        )
        .unwrap();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let RuntimeWorldPublicationOutcome::ProductUnpublished(effects) =
        port.execute_with_signal(prepared, &mut court.context(&root), &token, |_| {
            panic!("unsettled Relational cannot invoke Signal")
        })
    else {
        panic!("pending settlement remains partial")
    };
    assert_eq!(effects.cause(), ProductUnpublishedCause::SettlementPending);
    let handle = effects.recovery_handle();
    assert_ne!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .1
            .admission_identity(),
        root.basis().relational_basis().admission_identity()
    );
    court
        .world
        .recovery_port()
        .continue_effects(effects)
        .unwrap();
    let settled = court
        .world
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap();
    assert!(
        settled
            .component_results()
            .relational_settlement()
            .is_some(),
        "continuation completed the real owner settlement"
    );
    drop(settled);
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before
    );
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit()
    );
    court
        .world
        .recovery_port()
        .release_effects(&handle, 0)
        .unwrap();
    drop(root);
    court.finish();
}
