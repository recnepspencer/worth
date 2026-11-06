//! A foreign surviving selector remains source truth and forces a recomputation.
use super::super::{
    cold_owned_revision::{create, settle, RequiredProgram},
    correction_courts::install_definition,
    coupled::producer::handler,
    support_currentness::revise,
};
use super::*;
#[path = "retained.rs"]
mod retained;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn a_partial_shrink_recomputes_instead_of_refreshing_a_foreign_aggregate() {
    type S = currentness_guards::GuardModel;
    let host = install_definition::<S, RequiredProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let outcome = request
        .advanced()
        .mutate(Birth { id: 10 })
        .without_source()
        .idempotency(&"foreign-authored-member".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
        .outputs_of::<command::Outputs>()
        .unwrap()
        .entity::<command::Created<S>>()
        .unwrap()
        .entity_id();
    assert_eq!(
        revise(&host, &request, JOIN_ORBIT),
        1,
        "mixing preserves the own anchor"
    );
    assert_eq!(revise(&host, &request, RETIRE_ANCHOR),2,"the only surviving original member belongs to another publication, so changed source support must recompute");
    // orbit also records a walk, which independently forces the retry. Inspect the
    // retained aggregate itself so that walk invalidation cannot hide an own
    // postcondition incorrectly refreshed at the foreign surviving member.
    assert!(
        retained::aggregate_sizes(&host).contains(&2),
        "the mixed source observation must remain retained"
    );
    handler::reset_contacts();
    settle(&host, &request, 10);
    assert_eq!(handler::contacts(), 0, "the recomputed result is current");
}
