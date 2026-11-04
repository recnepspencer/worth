//! Every private roster and position kind has one Native writer.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    lifecycle::*,
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal, transactions::TransactionCommitError,
};
#[test]
fn ordinary_create_update_and_delete_of_each_current_metadata_kind_refuse_before_effect() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let body = create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let owner = body
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let capture = || {
        host.with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap()
    };
    let before = capture();
    let refusals = host
        .runtime()
        .probe_current_metadata_writes_for_test(body.product_branch(), owner, &scope)
        .unwrap();
    assert_eq!(
        refusals.len(),
        27,
        "three ordinary verbs for all nine reserved kinds"
    );
    for (kind, verb, result) in refusals {
        let error = result.expect_err("ordinary metadata writes never prepare");
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{kind:?}/{verb}: {error:?}")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::OrdinaryMetadataWrite),
            "{kind:?}/{verb}: {error:?}"
        );
    }
    let after = capture();
    host.with_host_advancement(|phase| {
        let before = host
            .runtime()
            .inspect_application_checkpoint(&phase, before.bytes())
            .unwrap();
        let after = host
            .runtime()
            .inspect_application_checkpoint(&phase, after.bytes())
            .unwrap();
        assert!(
            before.model == after.model,
            "ordinary metadata refusal changes no model value"
        );
        assert_eq!(before.live_records, after.live_records);
    })
    .unwrap();
    for id in [10, 20, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
}
