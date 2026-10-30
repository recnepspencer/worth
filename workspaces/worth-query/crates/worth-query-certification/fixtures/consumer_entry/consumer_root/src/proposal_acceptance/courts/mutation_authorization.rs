//! Current mutation grants authorize execution independently of both READ policies.
use super::{
    super::{
        application::{model::Model, storage::Storage},
        authentication::{self, Actor},
        host,
        mutation_access::{self, status},
        schema::data::AuthoredIntent,
    },
    retention_action::*,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

fn denied(
    denial: WorthQueryApplicationRequestMutationDenial,
    expected: WorthQueryOperationAuthorizationRefusalKind,
) {
    let WorthQueryApplicationRequestMutationDenial::Authorization(cause) = denial else {
        panic!("expected current mutation authorization refusal: {denial:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(expected)
    );
}

#[test]
fn retained_action_rechecks_mutation_authority_while_model_and_proposal_reads_stay_lawful() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 641, 7).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            642,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let request = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(model.branch());
    let operation = construct(&host, &model, &reviewer, edit(&model, 3), 643);
    operation.assess_current_authorization(&request).unwrap();
    complete(operation, &request, 3);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    let operation = construct(&host, &model, &reviewer, edit(&model, 4), 644);
    operation.assess_current_authorization(&request).unwrap();
    let status = |active| status::SetStatus {
        identity: mutation_access::grant_key("model-1", 2),
        active,
    };
    let before_administration = model.describe(&reviewer).unwrap();
    denied(
        request
            .advanced()
            .mutate(status(false))
            .without_source()
            .idempotency(&647)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_err(),
        WorthQueryOperationAuthorizationRefusalKind::PermissionDenied,
    );
    assert_eq!(model.describe(&reviewer).unwrap(), before_administration);
    operation.assess_current_authorization(&request).unwrap();
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    let change = |active, key| {
        host.request(&author.principal, author.scope)
            .on_branch(model.branch())
            .advanced()
            .mutate(status(active))
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    assert!(matches!(
        change(false, 645),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let before = model.describe(&reviewer).unwrap();
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    denied(
        operation
            .assess_current_authorization(&request)
            .unwrap_err(),
        WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing,
    );
    let WorthQuerySourceOnlyAdvance::Refused {
        operation,
        denial: WorthQuerySourceOnlyDenial::Mutation(cause),
    } = operation.advance(&request)
    else {
        panic!("execution must independently refuse and return original custody")
    };
    denied(
        cause,
        WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing,
    );
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    assert!(matches!(
        change(true, 646),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    operation.assess_current_authorization(&request).unwrap();
    complete(operation, &request, 4);
    assert_eq!(model.read(&reviewer).unwrap().days, 4);
    storage.load(&reviewer, &reference).unwrap();
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
