//! Native pin failures keep the read door's reason and canonical root.
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::{
    pin_runtime_read, read_plan::ReadPlan, WorthQueryApplicationReadExecutionDenialKind as ReadKind,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as OneShot;
use worth_relational::facade::indexes::DerivedIndexId;
#[derive(Clone, Copy)]
enum Case {
    Continuation,
    Target,
    Snapshot,
    Predicate,
}
#[test]
fn continuation_pin_keeps_its_missing_index_reason() {
    check(Case::Continuation);
}
#[test]
fn live_target_pin_keeps_its_missing_index_reason() {
    check(Case::Target);
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
                Case::Continuation | Case::Target => {
                    let expected = match case {
                        Case::Continuation => ReadKind::ContinuationIndexUnavailable,
                        Case::Target => ReadKind::TargetIdentityIndexUnavailable,
                        Case::Snapshot | Case::Predicate => unreachable!(),
                    };
                    let actual = world
                        .application
                        .primary_provider
                        .graph
                        .source_owner
                        .with_runtime(&request, |runtime| {
                            pin_runtime_read(
                                runtime,
                                &request,
                                &world.application.primary_provider.graph.layout,
                                &ReadPlan::of(&plan),
                                &[missing],
                                expected,
                            )
                            .unwrap_err()
                            .kind()
                        });
                    assert_eq!(actual, expected);
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
                        super::super::denial::session(cause, root.entity_id(), plan.query.name());
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
