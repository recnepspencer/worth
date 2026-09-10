use std::collections::BTreeMap;

use worth_query_installation::facade::TypedApplicationValue;
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
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    change_account_status(&world, account.entity_id(), "closed");

    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let denial = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .err()
        .expect("changed identity field must stale the resolved scope");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::StaleScope
    );
}

fn change_account_status(
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
        new_status.to_string().into_foundational_value(),
    )]));
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("stale-query-scope").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: account,
                fields,
            }),
        )),
    );
}
