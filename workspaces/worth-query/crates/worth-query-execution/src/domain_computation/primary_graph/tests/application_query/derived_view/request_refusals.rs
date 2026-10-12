//! Refusal precedes pair preparation; every request releases its transient memory.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use std::time::Instant;
use worth_execution::{CancellationSource, CancellationToken, ExecutionRequest};

#[test]
fn zero_memory_cancellation_and_expired_deadline_refuse_before_any_read() {
    let _owner = crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let root = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
    let membership_query = world
        .application
        .installed_schema()
        .certification_query(PublicAccountMembershipQuery::reference())
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let membership = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    selected
                        .retain_selection(&host_custody.borrowed_request_for_test())
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
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let cancelled = CancellationSource::new();
    cancelled.cancel();
    use crate::domain_computation::primary_graph::application_contribution::{
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    for workers in [None, Some(1), Some(2), Some(4)] {
        place(workers.map_or(Placement::Serial, |width| {
            Placement::Leased(std::num::NonZeroUsize::new(width).unwrap())
        }));
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                for index in 0..3 {
                    let memory = if index == 0 { 0 } else { 1 << 20 };
                    let lease = host_custody
                        .borrowed_request_for_test()
                        .in_scope(|parent| {
                            let parent = parent.expect("the active phase carries a lease");
                            parent
                                .child(worth_execution::LeaseRequest {
                                    policy: worth_foundational::ExecutionRequestPolicy::new(
                                        parent.policy().posture(),
                                        parent.policy().determinism(),
                                        worth_foundational::ExecutionBudget::new(
                                            std::num::NonZeroUsize::new(workers.unwrap_or(1))
                                                .unwrap(),
                                            memory,
                                            parent.policy().budget().work_ceiling(),
                                        ),
                                    ),
                                    cancellation: if index == 1 {
                                        cancelled.token()
                                    } else {
                                        CancellationToken::new()
                                    },
                                    deadline: (index == 2).then(Instant::now),
                                })
                                .unwrap()
                        })
                        .unwrap();
                    let request = ExecutionRequest::leased(&lease);
                    let view = world
                        .application
                        .open_managed_derived_collection_pair(
                            &ApplicationDerivedViewDefinition::new(
                                "request-refusal",
                                PublicAccountMembershipQuery::reference(),
                                ApplicationDerivedViewLimits::bounded(8, 32768),
                            ),
                            &membership_query,
                            &query,
                            &query,
                            selected.product(),
                        )
                        .unwrap();
                    let (witness, _scope) =
            crate::domain_computation::primary_graph::application_query::DispatchWitness::install(
                None,
            );
                    let mut prepared = 0;
                    let result = world
                        .application
                        .reconstruct_managed_derived_collection_pair(
                            request,
                            &view,
                            selected.product(),
                            &membership,
                            |row| {
                                row.members
                                    .iter()
                                    .map(|root| (*root, row.tag.clone()))
                                    .collect()
                            },
                            |_: &String| {
                                prepared += 1;
                                let first = selected
                                    .retain_selection(&host_custody.borrowed_request_for_test())
                                    .unwrap()
                                    .admit_application_query(
                                        &phase,
                                        &query,
                                        &access,
                                        ApplicationQueryParameterSet::new(),
                                        current_controls(&scope),
                                    )
                                    .unwrap();
                                let second = selected
                                    .retain_selection(&host_custody.borrowed_request_for_test())
                                    .unwrap()
                                    .admit_application_query(
                                        &phase,
                                        &query,
                                        &access,
                                        ApplicationQueryParameterSet::new(),
                                        current_controls(&scope),
                                    )
                                    .unwrap();
                                Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                            },
                            |row| row.status().to_owned(),
                            |row| row.status().to_owned(),
                            |_, second| SceneLabel(second.label().to_owned()),
                        );
                    let denial = result.expect_err("request must be refused");
                    match index {
                        0 => assert!(matches!(
                            denial,
                            WorthQueryManagedDerivedViewDenial::PolicyMemoryExhausted(
                                worth_execution::MemoryLimitDenial {
                                    admitted: 0,
                                    level: worth_execution::MemoryLimitLevel::Policy {
                                        ancestor: 0
                                    },
                                    ..
                                }
                            )
                        )),
                        1 => assert_eq!(denial, WorthQueryManagedDerivedViewDenial::Cancelled),
                        2 => {
                            assert_eq!(denial, WorthQueryManagedDerivedViewDenial::DeadlineElapsed)
                        }
                        _ => unreachable!(),
                    }
                    assert_eq!(prepared, 0);
                    assert!(witness.observations().0.is_empty());
                    drop(lease.reserve_memory(memory).unwrap());
                }
            })
            .expect("the fixture read owns its installed host call");
    }
}
