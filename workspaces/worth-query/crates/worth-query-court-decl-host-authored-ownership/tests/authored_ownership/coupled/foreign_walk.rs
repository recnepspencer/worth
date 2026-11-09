//! A mixed foreign walk remains source support during its reader's actual succession.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::{
    multi::{create, settle, TwoProgram},
    producer::handler,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn an_ordinary_successor_cannot_refresh_a_foreign_walk_it_changes() {
    type S = Model<64>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    create(&host, &request, 11, vec![]);
    settle(&host, &request, 10);
    settle(&host, &request, 11);
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
        .idempotency(&"invalidate-foreign-walker".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    handler::reset_contacts();
    settle(&host, &request, 10);
    assert_eq!(
        handler::contacts(),
        3,
        "adding and then removing the mixed foreign edge each requires a fresh source read"
    );
    handler::reset_contacts();
    settle(&host, &request, 10);
    assert_eq!(
        handler::contacts(),
        0,
        "the fresh foreign source observation now settles"
    );
}
