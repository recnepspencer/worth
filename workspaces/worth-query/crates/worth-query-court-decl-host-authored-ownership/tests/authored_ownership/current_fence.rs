//! An ordinary-edit head retains the same current custody across cold recovery.
use super::{
    correction_courts::install_definition,
    coupled::{
        history::reopen,
        multi::{create, settle, TwoProgram},
    },
    lifecycle::*,
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::publication::{AuthoredRetirementRefusal, PublicationError};
type S = Model<6>;

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
#[test]
fn ordinary_source_edit_head_preserves_warm_and_cold_fence_decisions() {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    create(&host, &request, 40, vec![]);
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let warm = denied(&host);
    let cold = denied(&reopen::<S>(checkpoint));
    assert_eq!(
        warm.authored_retirement_refusal(),
        cold.authored_retirement_refusal()
    );
    assert_eq!(warm.context.affected_records, cold.context.affected_records);
}
