use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::time::Duration;

use crate::domain_computation::primary_graph::application_attempt::Movement;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

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

mod root_path_guard_reads_its_pinned_truth_version;

#[test]
fn empty_indexed_root_set_stales_when_its_scoped_guard_becomes_a_match() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
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
            world
                .selected_product(&(phase).test_host_request())
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = super::installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let parameters = || {
        ApplicationQueryParameterSet::new()
            .bind(status_parameter(), "closed".to_string())
            .unwrap()
    };
    let initial = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let empty = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    initial
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(empty.rows().is_empty());
    assert!(empty.observed_sources().is_empty());
    let selection = empty.result_set_observation().selection_for_test();
    assert_eq!(selection.aspects.len(), 1);
    let guard = &selection.aspects[0];
    assert_eq!(guard.entity, account.entity_id());
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: guard.entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            guard.aspect.clone(),
            CanonicalFieldPath::single(guard.field.clone()),
        ),
        native_revision: guard.native_revision,
    };
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
    let current = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(!graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    fact.source_currentness_in(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        current.application_basis().snapshot_handle(),
                    )
                    .unwrap()
                    .unwrap()
                    .movement()
                        == Movement::Unmoved
                }))
        })
        .expect("the fixture read owns its installed host call");
    let matching = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    current
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(matching.rows().len(), 1);
    assert_ne!(
        empty.result_set_observation().idempotency_identity(),
        matching.result_set_observation().idempotency_identity()
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
                "open",
            )
        })
        .expect("the fixture read owns its installed host call");
    let reopened = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(!graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    fact.source_currentness_in(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        reopened.application_basis().snapshot_handle(),
                    )
                    .unwrap()
                    .unwrap()
                    .movement()
                        == Movement::Unmoved
                }))
        })
        .expect("the fixture read owns its installed host call");
    let reopened_result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    reopened
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(reopened_result.rows().is_empty());
    assert_eq!(reopened_result.rows(), empty.rows());
}

fn change_account_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        execution,
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
