//! An explicit coupled writer runs exactly once; Native grants it no next-round custody.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;

#[test]
fn an_explicit_coupled_writer_is_refused_a_continued_round() {
    type S = Model<8>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let first = create(&host, &request, 10, vec![20]);
    create(&host, &request, 11, vec![]);
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    super::producer::handler::reset_contacts();
    let _fault = host
        .runtime()
        .continue_next_coupled_route_with_its_effect_for_test();
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 20,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"continue-an-explicit-writer".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("an explicit coupled writer must not receive next-round custody")
    };
    let error = denial
        .authored_retirement_error()
        .expect("Native's typed route refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::CoupledContinuationRequiresEditSource),
        "{error:?}"
    );
    let native = first
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    assert_eq!(
        error.context.affected_records,
        [worth_relational::facade::transactions::RecordRef::Entity(
            native
        )]
    );
    assert_eq!(
        super::producer::handler::contacts(),
        1,
        "the one actual writer ran once and was not continued"
    );
    for id in [20, 101, 102, 111, 112] {
        assert!(
            request.query(query::Input { id }).execute().is_ok(),
            "the refusal changed {id}"
        );
    }
    for id in [103, 113] {
        assert!(
            request.query(query::Input { id }).execute().is_err(),
            "the refusal created {id}"
        );
    }
}
