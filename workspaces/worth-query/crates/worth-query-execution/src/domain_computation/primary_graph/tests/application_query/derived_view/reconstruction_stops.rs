//! Real reconstruction refuses native pin loss and interruption by value.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    NestedAccountQuery, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::WorthQueryDerivedPairReadPlans;
use worth_execution::ExecutionRequest;
use WorthQueryManagedDerivedViewDenial as Denial;

use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::provider_session::pin_control::PinControlGuardForTest;
use worth_query_declaration::facade::application_query::{
    ApplicationDerivedViewDefinition, ApplicationDerivedViewLimits,
};

#[test]
fn reconstruction_missing_predicate_snapshot_names_its_query_projection() {
    check(0);
}
#[test]
fn cancelled_pin_returns_the_stop_through_reconstruction() {
    check(1);
}
#[test]
fn the_same_reconstruction_reads_normally_without_a_pin_fault() {
    check(2);
}
fn check(case: usize) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let schema = world.application.installed_schema();
            let membership_query = schema
                .certification_query(PublicAccountMembershipQuery::reference())
                .unwrap();
            let first_query = schema
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let second_query = schema
                .certification_query(NestedAccountQuery::reference())
                .unwrap();
            assert_eq!(
                second_query
                    .read_family_binding()
                    .planning_contract()
                    .predicates()
                    .len(),
                1
            );
            let view = world
                .application
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "pin-refusal-reconstruction",
                        PublicAccountMembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 32768),
                    ),
                    &membership_query,
                    &first_query,
                    &second_query,
                    selected.product(),
                )
                .unwrap();
            let membership = world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    selected
                        .retain_selection(&execution)
                        .unwrap()
                        .admit_application_query(
                            &phase,
                            &membership_query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&scope),
                        )
                        .unwrap(),
                )
                .unwrap();
            let first = selected
                .retain_selection(&execution)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &first_query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap();
            let second = selected
                .retain_selection(&execution)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &second_query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    current_controls(&scope),
                )
                .unwrap();
            let mut plans = Some(WorthQueryDerivedPairReadPlans::new(first, second));
            let cancellation = worth_execution::CancellationSource::new();
            let child = execution
                .in_scope(|parent| {
                    let parent = parent.expect("the fixture lends host custody");
                    parent
                        .child(worth_execution::LeaseRequest {
                            policy: *parent.policy(),
                            cancellation: cancellation.token(),
                            deadline: None,
                        })
                        .unwrap()
                })
                .unwrap();
            let foreign =
                worth_relational::facade::runtime::RelationalRuntimeApi::builder().build();
            let unavailable = foreign
                .snapshots()
                .snapshot_for_observation(
                    &foreign
                        .observe_branch(&foreign.main_branch_identity())
                        .unwrap()
                        .1
                        .observation(),
                )
                .unwrap();
            let stimulus = if case == 1 {
                Some(PinControlGuardForTest::cancellation(cancellation))
            } else if case == 0 {
                // First side pins normally; second side is the one-predicate query.
                Some(PinControlGuardForTest::unavailable_snapshot(1, unavailable))
            } else {
                None
            };
            let actual = world
                .application
                .reconstruct_managed_derived_collection_pair(
                    ExecutionRequest::leased(&child),
                    &view,
                    selected.product(),
                    &membership,
                    |_| vec![(root.entity_id(), "open".to_owned())],
                    |_| Ok(plans.take().unwrap()),
                    |_| (),
                    |_| (),
                    |row, _| SceneLabel(row.label().to_owned()),
                );
            drop(stimulus);
            if case == 1 {
                assert_eq!(actual, Err(Denial::Cancelled));
            } else if case == 2 {
                assert_eq!(actual.unwrap().len(), 1);
            } else {
                let Err(Denial::ReadDenied {
                    root: refused,
                    denial,
                }) = actual
                else {
                    panic!("native snapshot loss must reach reconstruction: {actual:?}");
                };
                assert_eq!(refused, root.entity_id());
                // A one-predicate read selects its root without an index, so
                // the lost snapshot is the query's projection, not a predicate.
                assert_eq!(
                    denial.kind(),
                    WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable
                );
                assert_eq!(denial.subject(), second_query.name());
            }
        })
        .unwrap();
}
