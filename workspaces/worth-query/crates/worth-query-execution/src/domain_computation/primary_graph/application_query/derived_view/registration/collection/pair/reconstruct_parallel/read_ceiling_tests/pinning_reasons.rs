//! Native pin failures keep the read door's reason and canonical root.
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenialKind as ReadKind;
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as OneShot;
use worth_relational::facade::indexes::DerivedIndexId;
#[derive(Clone, Copy)]
enum Case {
    Panic,
    Snapshot,
    Predicate,
}
#[test]
fn continuation_pin_keeps_its_missing_index_reason() {
    missing_door_index(false, false);
}
#[test]
fn live_target_pin_keeps_its_missing_index_reason() {
    missing_door_index(true, false);
}
#[test]
fn pin_panic_does_not_poison_the_owner_mutex() {
    check(Case::Panic);
}
#[test]
fn continuation_pin_distinguishes_a_missing_predicate_from_its_own_index() {
    missing_door_index(false, true);
}
#[test]
fn worker_pin_keeps_snapshot_absence_at_the_root() {
    check(Case::Snapshot);
}
#[test]
fn worker_pin_keeps_index_absence_at_the_root() {
    check(Case::Predicate);
}
fn check(case: Case) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let request = host.borrowed_request_for_test();
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
            let query = world
                .application
                .installed_schema()
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let plan = selected
                .retain_selection(&request)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap();
            let missing = DerivedIndexId(u64::MAX);
            match case {
                Case::Panic => {
                    use crate::domain_computation::provider_session::pin_panic::PanicDuringPinGuardForTest;
                    let _panic = PanicDuringPinGuardForTest::install_on_this_thread();
                    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let _ = plan.graph_work.prepare_query_read(
                            plan.basis.identity(), &request, plan.basis.snapshot_handle(), |_| vec![],
                        );
                    }));
                    assert!(panic.is_err());
                    // A second real pin takes the same owner mutex after the unwind.
                    assert!(plan.graph_work.prepare_query_read(
                        plan.basis.identity(), &request, plan.basis.snapshot_handle(), |_| vec![],
                    ).is_ok());
                }
                Case::Snapshot | Case::Predicate => {
                    let other =
                        worth_relational::facade::runtime::RelationalRuntimeApi::builder().build();
                    let foreign = other
                        .snapshots()
                        .snapshot_for_observation(
                            &other
                                .observe_branch(&other.main_branch_identity())
                                .unwrap()
                                .1
                                .observation(),
                        )
                        .unwrap();
                    let snapshot = if matches!(case, Case::Snapshot) {
                        &foreign
                    } else {
                        plan.basis.snapshot_handle()
                    };
                    let ids = if matches!(case, Case::Predicate) {
                        vec![missing]
                    } else {
                        vec![]
                    };
                    let cause = match plan.graph_work.prepare_query_read(
                        plan.basis.identity(),
                        &request,
                        snapshot,
                        |_| ids,
                    ) {
                        Err(cause) => cause,
                        Ok(_) => panic!("the unavailable native source unexpectedly pinned"),
                    };
                    let outer =
                        super::super::denial::session(cause, root.entity_id(), &crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan::of(&plan));
                    let expected = if matches!(case, Case::Snapshot) {
                        OneShot::ProjectionUnavailable
                    } else {
                        OneShot::PredicateIndexUnavailable
                    };
                    match outer {
                        Denial::ReadDenied {
                            root: refused_root,
                            denial,
                        } => {
                            assert_eq!(refused_root, root.entity_id());
                            assert_eq!(denial.kind(), expected);
                        }
                        other => {
                            panic!("a pin refusal must keep the root's read denial: {other:?}")
                        }
                    }
                }
            }
        })
        .unwrap();
}

fn missing_door_index(live: bool, predicate_missing: bool) {
    use crate::domain_computation::primary_graph::application_query::read_execution::{
        read_continuation_page, read_live_target,
    };
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let request = host.borrowed_request_for_test();
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
                    AccountIdentity::reference(),
                    "account-1".into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let query = world
                .application
                .installed_schema()
                .certification_query(LiveAccountActivityQuery::reference())
                .unwrap();
            let mut plan = selected
                .admit_application_query_continuation(
                    &phase,
                    &query,
                    &access,
                    live_account_parameters("account-1"),
                    current_controls(&scope),
                )
                .unwrap();
            let graph = &world.application.primary_provider.graph;
            // Keep all installed bindings except the precise missing index.
            let mut layout = installed_layout();
            for (key, field) in layout.equality_fields_mut() {
                field.equality_index_id = graph
                    .layout
                    .equality_field(&key.0, &key.1, &key.2)
                    .unwrap()
                    .equality_index_id;
                if (live && key.0 == "Activity" && key.2 == "ActivityIdentity")
                    || (predicate_missing && key.0 == "Account" && key.2 == "AccountIdentity")
                {
                    field.equality_index_id = Some(DerivedIndexId(u64::MAX));
                }
            }
            if !live && !predicate_missing {
                assert!(plan.continuation_index_id.is_some());
                plan.continuation_index_id = Some(DerivedIndexId(u64::MAX));
            }

            let actual = graph.source_owner.with_runtime(&request, |runtime| {
                let outcome = if live {
                    read_live_target(
                        runtime,
                        &request,
                        &layout,
                        &plan,
                        root.identity_value().clone(),
                    )
                    .map(|_| ())
                } else {
                    read_continuation_page(runtime, &request, &layout, &plan, None).map(|_| ())
                };
                match outcome {
                    Err(denial) => denial.kind(),
                    Ok(_) => panic!("uninstalled index was read"),
                }
            });
            assert_eq!(
                actual,
                if predicate_missing {
                    ReadKind::PredicateIndexUnavailable
                } else if live {
                    ReadKind::TargetIdentityIndexUnavailable
                } else {
                    ReadKind::ContinuationIndexUnavailable
                }
            );
        })
        .unwrap();
}
