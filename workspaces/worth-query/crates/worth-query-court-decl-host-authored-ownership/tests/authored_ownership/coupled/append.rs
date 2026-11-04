//! Undo an actual append to a body that already has settled producer output.
use super::super::{
    correction_courts::{install_definition, reverse},
    lifecycle::*,
    *,
};
use super::{
    multi::{create, settle, TwoProgram},
    producer::handler,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;

#[test]
fn undo_of_an_append_to_a_settled_body_runs_one_coupled_successor() {
    type S = Model<60>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    handler::reset_contacts();
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: false,
            co_anchors: vec![],
            scope: 10,
            id: 30,
            depth: 0,
        })
        .without_source()
        .idempotency(&"append-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: appended, ..
    } = outcome
    else {
        panic!("append must commit: {outcome:?}")
    };
    settle(&host, &request, 10);
    assert_eq!(handler::contacts(), 1, "append settles one body successor");
    assert!(request.query(query::Input { id: 30 }).execute().is_ok());
    handler::reset_contacts();
    reverse(&request, &appended, "undo-append");
    assert_eq!(
        handler::contacts(),
        1,
        "undo contacts its coupled body exactly once"
    );
    assert!(request.query(query::Input { id: 30 }).execute().is_err());
    assert!(request.query(query::Input { id: 103 }).execute().is_err());
    assert!(request.query(query::Input { id: 104 }).execute().is_ok());
    settle(&host, &request, 10);
    assert_eq!(
        handler::contacts(),
        1,
        "the coupled successor is already settled"
    );
}
