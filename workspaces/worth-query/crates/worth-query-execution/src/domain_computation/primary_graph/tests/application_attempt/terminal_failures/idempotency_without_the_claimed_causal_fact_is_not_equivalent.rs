//! A real correction command's ordinary commit cannot impersonate its causal commit.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::correction_source;

#[test]
fn idempotency_without_the_claimed_causal_fact_is_not_equivalent() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "equivalence-source", "edited-label");
    let first = correction_source::prepare(&world, &source, "causal-equivalence");
    let retry = correction_source::prepare(&world, &source, "causal-equivalence");
    assert_eq!(first.binding, retry.binding);
    let ordinary =
        correction_source::committed(correction_source::publish_without_causality(&world, first));
    assert!(
        ordinary.aftermath_causality().is_none(),
        "the negative twin omits only the claimed causal fact"
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        correction_source::publish(&world, retry)
    else {
        panic!("a plain idempotency row cannot impersonate a causal commit");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}
