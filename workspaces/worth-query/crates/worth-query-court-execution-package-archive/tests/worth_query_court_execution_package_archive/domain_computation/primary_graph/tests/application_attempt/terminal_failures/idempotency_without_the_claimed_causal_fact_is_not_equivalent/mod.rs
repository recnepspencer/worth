//! A real correction command's ordinary commit cannot impersonate its causal commit.
use crate::domain_computation::primary_graph::tests::application_attempt::terminal_failures::*;

#[test]
fn idempotency_without_the_claimed_causal_fact_is_not_equivalent() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::install(worth_execution::MonotonicClock::system());
    let source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(&world, "equivalence-source", "edited-label");
    let first = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "causal-equivalence");
    let retry = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "causal-equivalence");
    assert_eq!(first.binding, retry.binding);
    let ordinary =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::committed(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish_without_causality(&world, first));
    assert!(
        ordinary.aftermath_causality().is_none(),
        "the negative twin omits only the claimed causal fact"
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish(&world, retry)
    else {
        panic!("a plain idempotency row cannot impersonate a causal commit");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}
