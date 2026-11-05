//! Editing only the second disconnected region must recontact its observing producer.
use super::super::super::{
    cold_owned_revision::{create, settle, RequiredProgram},
    correction_courts::install_definition,
    coupled::producer::handler,
    support_currentness::revise,
};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn a_retired_walk_root_keeps_both_surviving_regions_observed() {
    type S = currentness_guards::GuardModel;
    let host = install_definition::<S, RequiredProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    assert_eq!(
        revise(&host, &request, RETIRE_FORK_ROOT),
        1,
        "retiring the cut root settles with both live roots"
    );
    handler::reset_contacts();
    settle(&host, &request, 10);
    assert_eq!(handler::contacts(), 0);
    let outcome = request
        .advanced()
        .mutate(Extend { id: 10 })
        .without_source()
        .idempotency(&"extend-second-region".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
        .outputs_of::<command::Outputs>()
        .unwrap()
        .entity::<command::Created<S>>()
        .unwrap();
    assert_eq!(
        request
            .query(query::Input { id: 10 })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        RETIRE_FORK_ROOT,
        "the producer input did not change"
    );
    assert_eq!(
        handler::contacts(),
        0,
        "the independent edit does not demand producer outputs"
    );
    settle(&host, &request, 10);
    assert_eq!(
        handler::contacts(),
        1,
        "only the second surviving region changed, and its observer recontacts exactly once"
    );
    settle(&host, &request, 10);
    assert_eq!(
        handler::contacts(),
        1,
        "the refreshed second region then reuses"
    );
}
