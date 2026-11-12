//! An external delete of an owned member refuses at the current custody fence.
use super::{coupled::multi::TwoProgram, lifecycle::*, *};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::publication::{AuthoredRetirementRefusal, PublicationError};

pub(super) fn denied<T: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<T, TwoProgram>,
) -> PublicationError {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Plain {
            replacement: false,
            scope: 1,
            id: 101,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"external-current-custody-delete".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("external destruction must refuse: {outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("the original Native custody refusal")
        .clone();
    assert!(
        matches!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { .. })
        ),
        "{error:?}"
    );
    for id in [10, 40, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    error
}
