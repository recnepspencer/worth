//! A creation that omits its optional co-born kind reverses by its primary anchor alone.
use super::super::correction_courts::{install_program, reverse};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<73, Optional>;
#[test]
fn reversing_a_creation_without_its_optional_body_retires_the_primary_anchor_alone() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let create = |id: u64, create_co: bool, nonce: &str| {
        let outcome = request
            .advanced()
            .mutate(Create {
                create_co,
                co_anchors: vec![],
                scope: 1,
                id,
                depth: 0,
            })
            .without_source()
            .idempotency(&nonce.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        receipt
    };
    create(10, true, "body-birth");
    let alone = create(11, false, "node-under-existing-body");
    // The inverse retires node 11 and no body. Its own restoration demand names
    // the body key, which this retirement never changes: only the primary
    // anchor's fields are retained.
    reverse(&request, &alone, "undo-node-under-existing-body");
    assert!(request
        .query(super::super::query::Input { id: 11 })
        .execute()
        .is_err());
    assert!(request
        .query(super::super::query::Input { id: 10 })
        .execute()
        .is_ok());
    assert_eq!(
        request
            .query(query::Input { id: 10010 })
            .execute()
            .unwrap()
            .rows()[0]
            .id,
        10010
    );
}
