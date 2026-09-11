use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::current_controls;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, AccountIdentity, AccountStatus, CrossRootQuery,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryBasisPosture,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn root_path_guard_reads_its_pinned_truth_version() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
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
            AccountIdentity::reference(),
            "account-1".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(CrossRootQuery::reference())
        .unwrap();
    let pinned = world.selected_product();
    change_account_status(&world, account.entity_id(), "closed");
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let pinned_plan = pinned
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(10).unwrap(),
                NonZeroUsize::new(10_000).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let pinned_result = world
        .application
        .execute_application_query_one_shot(pinned_plan)
        .unwrap();
    let current_plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();
    let current_result = world
        .application
        .execute_application_query_one_shot(current_plan)
        .unwrap();

    assert_eq!(pinned_result.rows().len(), 2);
    assert!(current_result.rows().is_empty());
    assert_eq!(
        pinned_result.receipt().basis_posture(),
        WorthQueryApplicationQueryBasisPosture::SelectedProduct
    );
}

fn change_account_status(
    world: &super::super::fixture::AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
    status: &str,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountStatus::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&status.to_string()).unwrap(),
    )]));
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("change-status-after-product-selection").push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: account,
                    fields,
                },
            )),
        ),
    );
}
