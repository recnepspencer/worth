//! The #277 typed limit's conversion (PR C C4 r6): a successor round whose
//! own-output view was found incoherent stops typed (`PriorEffectRead`) only
//! when its prepare failed untyped (`ProducerUnavailable`), the failure the
//! incoherent read becomes. Every typed stop keeps its own kind, even with
//! the view marked incoherent.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::{
    admission_converts_to_prior_effect_for_test as admission_converts,
    converts_to_prior_effect_for_test as converts,
};
use worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind as Kind;

#[test]
fn only_the_untyped_producer_failure_converts() {
    assert!(converts(Kind::ProducerUnavailable));
}

#[test]
fn cancellation_budget_and_domain_stops_keep_their_kind() {
    for kind in [
        Kind::Cancelled,
        Kind::WorkBudgetExceeded,
        Kind::ProducerDomainDenied,
        Kind::CurrentControlBusy,
    ] {
        assert!(!converts(kind.clone()), "{kind:?}");
    }
}

#[test]
fn a_request_admission_stop_never_converts() {
    assert!(!admission_converts());
}
