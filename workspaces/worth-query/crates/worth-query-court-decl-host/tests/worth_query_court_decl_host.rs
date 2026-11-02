//! Query proofs grouped by their product frontier.
#![recursion_limit = "512"]

#[path = "worth_query_court_decl_host/certification/mod.rs"]
mod certification;

#[path = "worth_query_court_decl_host/aspect_patterns/mod.rs"]
mod aspect_patterns;

#[path = "worth_query_court_decl_host/cost/mod.rs"]
mod cost;

#[path = "worth_query_court_decl_host/determinism/mod.rs"]
mod determinism;

#[path = "worth_query_court_decl_host/effect_order/mod.rs"]
mod effect_order;

#[test]
fn inspection_refusal_types_are_nameable_and_comparable_through_host() {
    use worth_query_host::facade::causal_inspection::*;
    fn diagnostic<T: Clone + std::fmt::Debug + PartialEq>() {}
    diagnostic::<OutputFieldCaptureFailure>();
    diagnostic::<StoreDenial>();
    diagnostic::<IndexedSelectionReobserveDenial>();
    diagnostic::<OwnEffectOnReads>();
    diagnostic::<RebaseVerificationReason>();
    diagnostic::<WorthQueryApplicationDisclosureContractDenial>();
    diagnostic::<WorthQueryManagedGraphWorkSessionStartDenial>();
}
