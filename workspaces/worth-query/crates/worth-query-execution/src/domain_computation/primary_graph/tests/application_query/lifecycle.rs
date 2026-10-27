use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

pub(super) use super::lifecycle_mutations::{disable_mapping, revoke_account_ownership};
use super::{current_controls, installed_query};
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_account_parameters, status_parameter, AccountIdentity,
        AccountStatus,
    },
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

mod retained_continuation_releases_the_query_registry_basis;

#[test]
fn foreign_runtime_rejects_plan_and_releases_its_basis() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
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
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
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
                        .expect("fixture query parameter must encode"),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let basis = plan.basis_identity().clone();
    assert!(basis_is_live(&world, &basis));

    let denial = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("foreign runtime cannot consume a sealed plan")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::ForeignPlan
    );
    assert!(!basis_is_live(&world, &basis));
}

#[test]
fn exact_query_observation_reuses_its_live_snapshot_for_fresh_security_stages() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
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
            selected
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
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let query = installed_query(&world);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let plan = world
        .application
        .with_host_advancement(|phase| {
            selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(observer.observe().acquisitions(), before.acquisitions());
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.rows().len(), 1);
    assert!(result.receipt().basis_released());
    assert_eq!(observer.observe().acquisitions(), before.acquisitions());
    assert_eq!(observer.observe().active(), before.active() - 1);

    let source = result.into_admitted_disclosed().into_output_demand_source();
    let observation = source
        .retain_inspection_observation()
        .expect("a completed query retains its genuine result-set source");
    let retained = observation.clone();
    drop(source);
    assert_eq!(observer.observe().active(), before.active() - 1);
    assert_eq!(observer.observe().acquisitions(), before.acquisitions());
    observation.close();
    retained.close();
    assert_eq!(observer.observe().active(), before.active() - 1);
}

#[test]
fn cancellation_after_admission_releases_basis_before_projection() {
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
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
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
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
                        .expect("fixture query parameter must encode"),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let basis = plan.basis_identity().clone();

    assert!(basis_is_live(&world, &basis));
    cancellation.cancel();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("cancelled request cannot enter provider execution")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Cancelled
    );
    assert!(!basis_is_live(&world, &basis));
}

mod principal_revocation_after_product_admission_denies_before_proje;

mod scope_revocation_after_product_admission_denies_before_projectio;

pub(super) fn basis_is_live(
    world: &super::super::fixture::AuthorizationWorld,
    _basis: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
) -> bool {
    world
        .application
        .application_query_basis_observer()
        .observe()
        .active()
        > 0
}
