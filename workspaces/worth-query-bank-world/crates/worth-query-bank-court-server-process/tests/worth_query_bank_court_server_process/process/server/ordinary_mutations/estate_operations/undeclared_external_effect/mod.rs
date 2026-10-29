pub(crate) use worth_query_bank_worlds::facade::server::estate_operations::undeclared_external_effect::*;

#[test]
fn an_undeclared_external_effect_costs_no_outbox_and_no_dispatch() {
    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let fixture = ordinary_read_world("undeclared-external-effect", 0);
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("the bank installs its rail once per runtime");
    assert!(fixture.world.runtime.has_external_effect_transport());

    let owner = fixture.authenticate(OWNER);
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: fixture.personal_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(250).unwrap(),
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("undeclared-effect-send").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("send construction: {cause:?}"));
    let scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        outcome,
    ) = outcome.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("fresh retained send completion")
    };

    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        receipt,
        ..
    } = outcome.evidence()
    else {
        panic!("fresh native evidence")
    };

    assert!(
        receipt.dispatch_outbox().is_none(),
        "an operation with no declared external effect writes no outbox row"
    );
    assert!(receipt.external_dispatch().is_none());
    assert!(
        receipt.dispatch_outbox().is_none(),
        "an undeclared effect has no committed outbox to observe"
    );
    let dispatch_work = receipt.canonical_work().external_dispatch();
    assert_eq!(dispatch_work.basis_preparations(), 0);
    assert_eq!(dispatch_work.digest_derivations(), 0);
    assert_eq!(dispatch_work.canonical_encoded_bytes(), 0);
    assert_eq!(dispatch_work.digest_text_materializations(), 0);
    assert!(
        transport.attempts().is_empty(),
        "the installed rail must never hear from an undeclared effect"
    );
    assert!(outcome.close(&request).is_ok());
}
