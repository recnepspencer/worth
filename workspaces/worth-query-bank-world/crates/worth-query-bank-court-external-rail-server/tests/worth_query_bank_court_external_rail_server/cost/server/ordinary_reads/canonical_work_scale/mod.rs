pub(crate) use worth_query_bank_testkit_server::facade::ordinary_reads::canonical_work_scale::*;

#[test]
fn result_and_graph_fanout_is_visible_only_as_closed_public_work() {
    let fixture = canonical_scale_world();
    let baseline_actor = fixture.authenticate_baseline();
    let expanded_actor = fixture.authenticate_expanded();

    let baseline_result = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&baseline_actor)
        .controls(controls(192))
        .execute()
        .expect("baseline account discovery should execute");
    let expanded_result = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&expanded_actor)
        .controls(controls(192))
        .execute()
        .expect("expanded account discovery should execute");

    assert_eq!(baseline_result.rows().len(), 1);
    assert_eq!(expanded_result.rows().len(), 192);
    let baseline = baseline_result.receipt().inspect();
    let expanded = expanded_result.receipt().inspect();
    assert!(expanded.ordinary_work_units() > baseline.ordinary_work_units());
    assert!(baseline.terminal_resources_released());
    assert!(expanded.terminal_resources_released());
}

#[test]
fn guarded_candidate_fanout_is_visible_only_as_closed_public_work() {
    const EXPANDED_CANDIDATE_COUNT: usize = 64;

    let fixture = ordinary_read_world_with_pending_payments("candidate-scale", 0, 1);
    let approver = fixture.authenticate(APPROVER);
    let owner = fixture.authenticate(OWNER);

    let baseline_result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(EXPANDED_CANDIDATE_COUNT))
        .execute()
        .expect("baseline guarded query should execute");

    for ordinal in 1..EXPANDED_CANDIDATE_COUNT {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&owner, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::initiate_business_payment(
                InitiateBusinessPayment {
                    business: BusinessId::new(1).unwrap(),
                    from: fixture.business_account,
                    recipient: principal_id(RECIPIENT),
                    amount: Money::from_minor(1).unwrap(),
                },
            ))
            .as_principal(&owner)
            .controls(BankMutationControls::new(
                request_scope(),
                BankIdempotencyKey::new(format!("candidate-scale-{ordinal}")).unwrap(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("declared initiation: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("scale initiation {ordinal} completes")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("scale close: {cause:?}"));
    }

    let expanded_result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(EXPANDED_CANDIDATE_COUNT))
        .execute()
        .expect("expanded guarded query should execute");

    assert_eq!(baseline_result.rows().len(), 1);
    assert_eq!(expanded_result.rows().len(), EXPANDED_CANDIDATE_COUNT);
    assert!(
        expanded_result.receipt().inspect().ordinary_work_units()
            > baseline_result.receipt().inspect().ordinary_work_units()
    );
}

#[test]
fn policy_fact_fanout_does_not_leak_into_closed_disclosure_evidence() {
    let fixture = canonical_scale_world();
    let baseline_actor = fixture.authenticate_baseline();
    let expanded_actor = fixture.authenticate_expanded();

    let baseline_result = fixture
        .world
        .runtime
        .account_activity(fixture.baseline_account)
        .as_principal(&baseline_actor)
        .execute(BankReadControls::current(request_scope(), 1).unwrap())
        .expect("baseline guarded account query should execute");
    let expanded_result = fixture
        .world
        .runtime
        .account_activity(fixture.expanded_account)
        .as_principal(&expanded_actor)
        .execute(BankReadControls::current(request_scope(), 1).unwrap())
        .expect("expanded guarded account query should execute");

    assert_eq!(baseline_result.rows().len(), 1);
    assert_eq!(expanded_result.rows().len(), 1);
    let baseline = baseline_result.receipt().disclosure();
    let expanded = expanded_result.receipt().disclosure();
    assert_eq!(expanded.posture(), baseline.posture());
    assert_eq!(
        expanded.disclosure_decision_count(),
        baseline.disclosure_decision_count()
    );
    assert_eq!(
        expanded.disclosed_value_count(),
        baseline.disclosed_value_count()
    );
    assert_eq!(
        expanded.omitted_value_count(),
        baseline.omitted_value_count()
    );
    assert_eq!(
        expanded.authorization_decision_fact_count(),
        baseline.authorization_decision_fact_count()
    );
    assert_eq!(expanded.identity(), baseline.identity());
}
