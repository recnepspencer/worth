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
        installed_authorization_world, status_parameter, AccountIdentity, AccountStatus,
        CrossRootQuery,
    },
    WorthQueryApplicationObservedFact, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryBasisPosture, WorthQueryPrincipalResolutionMode,
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
            &external,
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
        .certification_query(CrossRootQuery::reference())
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
    let empty_selection = current_result.result_set_observation().selection_for_test();
    assert_eq!(empty_selection.aspects.len(), 1);
    assert_eq!(empty_selection.entities, [account.entity_id()]);
    let selection = pinned_result.observed_sources()[0]
        .footprint_for_test()
        .root_selection
        .as_ref()
        .expect("pinned path result retains the guard source");
    let guard = selection
        .aspects
        .iter()
        .find(|aspect| aspect.entity == account.entity_id())
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAspectRevision {
        entity_id: guard.entity,
        aspect: guard.aspect.clone(),
        native_revision: guard.native_revision,
    };
    let current = world.selected_product();
    let graph = world.application.runtime.primary_graph().unwrap();
    assert!(
        !graph.integration_handle().with_runtime(|runtime| {
            fact.source_currentness_in(runtime, current.application_basis().snapshot_handle(), 1)
                .unwrap()
                .0
        }),
        "changed root-path guard must stale the selected source"
    );
    assert_eq!(
        pinned_result.receipt().basis_posture(),
        WorthQueryApplicationQueryBasisPosture::SelectedProduct
    );
    let absent_guard = &empty_selection.aspects[0];
    let absent_fact = WorthQueryApplicationObservedFact::SourceAspectRevision {
        entity_id: absent_guard.entity,
        aspect: absent_guard.aspect.clone(),
        native_revision: absent_guard.native_revision,
    };
    change_account_status(&world, account.entity_id(), "open");
    let reopened = world.selected_product();
    assert!(
        !graph.integration_handle().with_runtime(|runtime| {
            absent_fact
                .source_currentness_in(runtime, reopened.application_basis().snapshot_handle(), 1)
                .unwrap()
                .0
        }),
        "a matching guard edit must stale an empty result-set observation"
    );
    let reopened_result = world
        .application
        .execute_application_query_one_shot(
            reopened
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(reopened_result.rows().len(), 2);
    assert_ne!(
        current_result
            .result_set_observation()
            .idempotency_identity(),
        reopened_result
            .result_set_observation()
            .idempotency_identity()
    );
}

#[test]
fn empty_indexed_root_set_stales_when_its_scoped_guard_becomes_a_match() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .selected_product()
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = super::installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let parameters = || {
        ApplicationQueryParameterSet::new()
            .bind(status_parameter(), "closed".to_string())
            .unwrap()
    };
    let initial = world.selected_product();
    let empty = world
        .application
        .execute_application_query_one_shot(
            initial
                .admit_application_query(&query, &access, parameters(), current_controls(&request))
                .unwrap(),
        )
        .unwrap();
    assert!(empty.rows().is_empty());
    assert!(empty.observed_sources().is_empty());
    let selection = empty.result_set_observation().selection_for_test();
    assert_eq!(selection.aspects.len(), 1);
    let guard = &selection.aspects[0];
    assert_eq!(guard.entity, account.entity_id());
    let fact = WorthQueryApplicationObservedFact::SourceAspectRevision {
        entity_id: guard.entity,
        aspect: guard.aspect.clone(),
        native_revision: guard.native_revision,
    };
    change_account_status(&world, account.entity_id(), "closed");
    let current = world.selected_product();
    let graph = world.application.runtime.primary_graph().unwrap();
    assert!(!graph.integration_handle().with_runtime(|runtime| {
        fact.source_currentness_in(runtime, current.application_basis().snapshot_handle(), 1)
            .unwrap()
            .0
    }));
    let matching = world
        .application
        .execute_application_query_one_shot(
            current
                .admit_application_query(&query, &access, parameters(), current_controls(&request))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(matching.rows().len(), 1);
    assert_ne!(
        empty.result_set_observation().idempotency_identity(),
        matching.result_set_observation().idempotency_identity()
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
