use std::time::Instant;

use bank_domain::model::{BusinessId, Money};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::InitiateBusinessPayment;
use bank_server::{mutations, queries, BankMutationControls, BankMutationStatus, BankReadControls};

use super::canonical_scale_fixture::canonical_scale_world;
use super::fixture::{
    ordinary_read_world_with_pending_payments, over_budget_discovery_world, principal_id, APPROVER,
    OWNER, RECIPIENT,
};
use crate::support::request_scope;

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
        let outcome = fixture
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
            .execute();
        assert!(
            matches!(outcome.status(), BankMutationStatus::Committed(_)),
            "scale mutation {ordinal} must commit: {outcome:?}"
        );
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
        .execute(BankReadControls::current(request_scope(), 1, 100_000).unwrap())
        .expect("baseline guarded account query should execute");
    let expanded_result = fixture
        .world
        .runtime
        .account_activity(fixture.expanded_account)
        .as_principal(&expanded_actor)
        .execute(BankReadControls::current(request_scope(), 1, 100_000).unwrap())
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

#[test]
#[ignore = "scheduled high-operation speed probe; run explicitly with --ignored --nocapture"]
fn high_operation_speed_probe_keeps_publication_work_stable() {
    const QUERY_COUNT: usize = 512;
    const RESULT_COUNT: usize = 64;

    let fixture = over_budget_discovery_world("canonical-work-high-operation", RESULT_COUNT);
    let actor = fixture.authenticate();
    let started = Instant::now();
    let mut expected = None;
    let mut observed_rows = 0usize;

    for _ in 0..QUERY_COUNT {
        let result = fixture
            .world
            .runtime
            .query(queries::accounts())
            .as_principal(&actor)
            .controls(controls(RESULT_COUNT))
            .execute()
            .expect("high-operation account discovery should execute");
        observed_rows = observed_rows
            .checked_add(result.rows().len())
            .expect("observed row count remains bounded");
        let inspection = result.receipt().inspect();
        let phases = (
            inspection.publication_canonical_entries(),
            inspection.publication_sha256_compression_blocks(),
            inspection.publication_identity_text_materializations(),
        );
        if let Some(expected) = expected {
            assert_eq!(phases, expected);
        } else {
            expected = Some(phases);
        }
    }

    let elapsed = started.elapsed();
    let seconds = elapsed.as_secs_f64();
    let queries_per_second = QUERY_COUNT as f64 / seconds;
    let rows_per_second = observed_rows as f64 / seconds;
    assert_eq!(observed_rows, QUERY_COUNT * RESULT_COUNT);
    eprintln!(
        "profile={} queries={QUERY_COUNT} rows={observed_rows} elapsed={elapsed:?} \
         queries_per_second={queries_per_second:.2} rows_per_second={rows_per_second:.2}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
}

fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results, 100_000)
        .expect("high-operation controls are bounded")
}
