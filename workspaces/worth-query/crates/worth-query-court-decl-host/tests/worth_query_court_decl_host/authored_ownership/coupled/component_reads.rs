//! The body reads the surviving authored stage component through the actual candidate.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn a_coupled_body_reads_the_stage_component_after_the_authored_prefix() {
    type S = Model<21>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20, 30]);
    settle(&host, &request, 10);
    super::producer::handler::reset_contacts();
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
        .idempotency(&"retire-stage-with-component-reader".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(super::producer::handler::contacts(), 1);
    for id in [20, 102] {
        assert!(request.query(query::Input { id }).execute().is_err());
    }
    for id in [10, 30, 101, 103] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    settle(&host, &request, 10);
    assert_eq!(
        super::producer::handler::contacts(),
        1,
        "unchanged stage component reuses the settled body"
    );
}

#[test]
fn a_coupled_body_keeps_component_support_after_its_own_cell_succession() {
    own_output_support::<22>();
}
#[test]
fn a_coupled_body_keeps_derived_support_after_its_own_cell_succession() {
    own_output_support::<23>();
}
#[test]
fn a_coupled_body_keeps_a_derived_component_after_its_own_cell_succession() {
    own_output_support::<26>();
}
fn own_output_support<const MODE: u8>() {
    type S<const MODE: u8> = Model<MODE>;
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    super::producer::handler::reset_contacts();
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
        .idempotency(&"retire-stage-own-component-reader".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(super::producer::handler::contacts(), 1);
    for id in [20, 102] {
        assert!(request.query(query::Input { id }).execute().is_err());
    }
    for id in [10, 101, 103] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
}
