//! A cascade cannot turn an explicitly supplied private deletion into Native standing.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};
#[test]
fn caller_supplied_private_membership_delete_refuses_inside_the_real_cascade() {
    type S = Model<14>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let _fault = host.runtime().supply_private_membership_deletion_for_test();
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 20,
            co_anchors: vec![],
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"explicit-private-delete".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("the original typed private-write refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::OrdinaryMetadataWrite),
        "{error:?}"
    );
    assert!(matches!(
        error.context.affected_records.as_slice(),
        [RecordRef::Relation(_)]
    ));
    for id in [10, 20, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    assert!(request.query(query::Input { id: 103 }).execute().is_err());
    settle(&host, &request, 10);
}
