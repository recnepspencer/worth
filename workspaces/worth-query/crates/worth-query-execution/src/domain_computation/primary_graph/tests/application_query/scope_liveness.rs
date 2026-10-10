use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::*;

#[test]
fn changed_identity_field_makes_resolved_scope_stale_before_admission() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
                "closed",
            )
        })
        .expect("the fixture read owns its installed host call");

    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .err()
                .expect("changed identity field must stale the resolved scope")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::StaleScope
    );
}

fn change_account_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
    new_status: &str,
) {
    let graph = world
        .application
        .runtime
        .primary_graph()
        .expect("test world publishes a primary graph");
    let status_field = AccountStatus::reference();
    let locator = graph
        .layout
        .field_locator(
            status_field.entity(),
            status_field.aspect(),
            status_field.field(),
        )
        .expect("account status is installed")
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&new_status.to_string()).unwrap(),
    )]));
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("stale-query-scope").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: account,
                fields,
            }),
        )),
    );
}
