//! Real admitted pair reads across request backings; projection stays on the owner.
pub use crate::domain_computation::primary_graph::tests::application_query::derived_view::*;
pub use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::Case;
pub use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::Observation;
pub use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::ScopeInterruption;

pub use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::NestedAccountQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PublicAccountMembershipQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PublicScopedAccountSummaryQuery;
pub use std::num::NonZeroUsize;
pub use worth_execution::ExecutionRequest;
pub use worth_execution::ExecutionWorkCeiling;
pub use worth_execution::LeaseRequest;

pub(super) use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::default_case::observe;
pub fn observe_case(
    workers: Option<usize>,
    count: usize,
    duplicate: bool,
    owner_failure: bool,
    failures: &[usize],
    case: Case,
) -> Observation {
    let _run = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    place(workers.map_or(Placement::Serial, |width| {
        Placement::Leased(NonZeroUsize::new(width).unwrap())
    }));
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
    let host_request = phase.test_host_request();
    let phase_custody = &host_request;
    let request_execution = phase.execution_request_for(world.application.product_runtime()).expect("the pair fixture lends its installed phase");
    let execution = &request_execution;

    if count == 3 {
        crate::domain_computation::primary_graph::tests::application_query::derived_view::reconstruction_population::add_third_root(execution, &world);
    }
    if count >= 2 {
        crate::domain_computation::primary_graph::tests::application_query::derived_view::reconstruction_population::add_primary_reads(
            &phase_custody.phase_for_application(&world.application),
            execution,
            &world,
            count == 3,
        );
    }
    let query_cancel =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let scope = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        Some(
            std::time::Instant::now()
                + Duration::from_secs(
                    if matches!(case.query_interruption, Some(ScopeInterruption::Deadline)) {
                        5
                    } else {
                        60
                    },
                ),
        ),
        query_cancel.token(),
    );
    let caller_cancel = worth_execution::CancellationSource::new();
    let selected = world.selected_product(phase_custody);
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = selected
        .resolve_authenticated_principal(
            &phase_custody.phase_for_application(&world.application),
            &world.binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let roots = ["open", "unrelated", "third"]
        .into_iter()
        .take(count.max(1))
        .map(|status| {
            selected
                .resolve_entity(
                    &phase_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    status.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let access = roots
        .iter()
        .map(|root| WorthQueryApplicationQueryAccessContext::new(&principal, root))
        .collect::<Vec<_>>();
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
    let second_query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let view = world
        .application
        .open_managed_derived_collection_pair(
            &ApplicationDerivedViewDefinition::new(
                "leased-reconstruction",
                PublicAccountMembershipQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 32768),
            ),
            &membership_query,
            &query,
            &second_query,
            selected.product(),
        )
        .unwrap();
    let membership = world
        .application
        .execute_application_query_one_shot(
            &phase_custody.phase_for_application(&world.application),
            selected
                .retain_selection(execution)
                .unwrap()
                .admit_application_query(
                    &phase_custody.phase_for_application(&world.application),
                    &membership_query,
                    &access[0],
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap(),
        )
        .unwrap();
    // Execution has a real immutable deadline. Mint it after world setup,
    // allow preparation five seconds, and require a fresh selected rendezvous.
    // The kernel waits for expiry there; elapsed setup can only fail the test.
    let deadline = case
        .interrupt
        .filter(|(_, deadline)| *deadline)
        .map(|_| std::time::Instant::now() + Duration::from_secs(5));
    let lease = execution.in_scope(|parent| {
        { let parent = parent.expect("the active phase is leased"); parent.child(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(parent.policy().posture(), parent.policy().determinism(), worth_foundational::ExecutionBudget::new(NonZeroUsize::new(workers.unwrap_or(1)).unwrap(), None, parent.policy().work_ceiling())),
            cancellation: caller_cancel.token(),
            deadline,
        }).unwrap() }
    }).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let execution = &request;
    let owner = std::thread::current().id();
    let mut canonical_roots = roots
        .iter()
        .map(|root| root.entity_id())
        .collect::<Vec<_>>();
    canonical_roots.sort();
    canonical_roots.truncate(count);
    let later = (count >= 2
        && workers.is_some_and(|width| width > 1)
        && !duplicate
        && case.interrupt.is_none()
        && case.query_interruption.is_none())
    .then(|| canonical_roots[1]);
    let (witness, _witness_scope) =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::DispatchWitness::install(
            later,
        );
    if let Some((index, is_deadline)) = case.interrupt {
        let root = canonical_roots[index];
        let earlier = canonical_roots[..index].to_vec();
        use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::Interrupt;
        witness.interrupt(if is_deadline {
            Interrupt::Deadline {
                root,
                earlier,
                deadline: deadline.unwrap(),
            }
        } else {
            Interrupt::Cancel {
                root,
                earlier,
                source: caller_cancel.clone(),
            }
        });
    } else if let Some(interruption) = case.query_interruption {
        use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::Interrupt;
        let root = *canonical_roots.last().unwrap();
        let earlier = canonical_roots[..count - 1].to_vec();
        witness.interrupt(match interruption {
            ScopeInterruption::Cancel => Interrupt::QueryCancel {
                root,
                earlier,
                source: query_cancel,
            },
            ScopeInterruption::Deadline => Interrupt::QueryDeadline {
                root,
                earlier,
                deadline: scope
                    .deadline()
                    .expect("this case supplies a query deadline"),
            },
        });
    }
    let mut projected = Vec::new();
    let mut prepared = 0;
    let mut run = |failures: &[usize], owner_failure: bool| {
        projected.clear();
        prepared = 0;
        witness.reset();
        let result = request.run(ExecutionWorkCeiling::new(None), |_| {
            world
                .application
                .reconstruct_managed_derived_collection_pair(
                    request,
                    &view,
                    selected.product(),
                    &membership,
                    |_| {
                        // The caller declares members in reverse order; Query owns ordering.
                        let mut members = (0..count)
                            .rev()
                            .map(|index| {
                                (
                                    roots[index].entity_id(),
                                    ["open", "unrelated", "third"][index].to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();
                        if duplicate {
                            members.push(members[0].clone());
                        }
                        members
                    },
                    |member: &String| {
                        assert_eq!(std::thread::current().id(), owner);
                        prepared += 1;
                        let index = ["open", "unrelated", "third"]
                            .iter()
                            .position(|status| *status == member)
                            .unwrap();
                        let first = selected
                            .retain_selection(execution)
                            .unwrap()
                            .admit_application_query(
                                &phase_custody.phase_for_application(&world.application),
                                &query,
                                &access[index],
                                ApplicationQueryParameterSet::new(),
                                current_controls(&scope),
                            )
                            .unwrap();
                        let parameter = if failures.contains(&index) {
                            "missing".to_owned()
                        } else {
                            member.clone()
                        };
                        let second = selected
                            .retain_selection(execution)
                            .unwrap()
                            .admit_application_query(
                                &phase_custody.phase_for_application(&world.application),
                                &second_query,
                                &access[index],
                                ApplicationQueryParameterSet::new()
                                    .bind(status_parameter(), parameter)
                                    .unwrap(),
                                current_controls(&scope),
                            )
                            .unwrap();
                        Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                    },
                    |_| 11_u64,
                    |row| row.primary_sequence(),
                    |first, second| {
                        assert_eq!(std::thread::current().id(), owner);
                        assert_eq!(second.primary_sequence(), 11);
                        assert_eq!(
                            second.all_sequences(),
                            if first.status() == "open" {
                                &[11, 22][..]
                            } else {
                                &[][..]
                            }
                        );
                        if owner_failure {
                            panic!("owner projection failure");
                        }
                        let value = format!("{}:{:?}", first.label(), second.all_sequences());
                        projected.push((
                            roots[["open", "unrelated", "third"]
                                .iter()
                                .position(|status| *status == first.status())
                                .unwrap()]
                            .entity_id(),
                            value.clone(),
                        ));
                        SceneLabel(value)
                    },
                )
        });
        witness.assert_interruption_point();
        result.unwrap()
    };
    let prior = if case.warm {
        let (result, _) = run(&[], false);
        result.unwrap()
    } else {
        Vec::new()
    };
    let snapshot = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    let prior_values = prior
        .iter()
        .map(|key| snapshot.get(key).unwrap().unwrap().0.clone())
        .collect::<Vec<_>>();
    let (result, report) = run(failures, owner_failure);
    let (entries, completions) = witness.observations();
    let published_values = result.as_ref().ok().map(|keys| {
        let snapshot = world
            .application
            .observe_managed_derived_view(&view)
            .unwrap();
        keys.iter()
            .map(|key| snapshot.get(key).unwrap().unwrap().0.clone())
            .collect()
    });
    // Owner finalization retains its returned denial's payload. Every other hold
    // must release: the payload plus this probe fills the entire selected limit.
    let denial_bytes =
        if let Err(error @ WorthQueryManagedDerivedViewDenial::ReadDenied { .. }) = &result {
            let name = if case.query_interruption.is_some() {
                query.name()
            } else {
                second_query.name()
            };
            crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::denial_custody::check(error, name);
            crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::denial_custody::bytes(name)
        } else {
            0
        };
    let available = (16_u64 << 20).checked_sub(denial_bytes).unwrap();
    drop(lease.reserve_memory(available).unwrap());
    let retained_values = prior
        .iter()
        .map(|key| snapshot.get(key).unwrap().unwrap().0.clone())
        .collect::<Vec<_>>();
    let retention = view.reconstruction_retention_for_test();
    crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::verification::verify(
        Observation {
            interruption_point: witness.interruption_point(),
            dependencies: witness.dependencies(),
            retention,
            roots: canonical_roots,
            map: witness.map_observation(),
            retained_values,
            values: projected.iter().map(|(_, value)| value.clone()).collect(),
            work: report.charged_work(),
            denial: result.err(),
            prepared,
        },
        crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::verification::ApplicationEvidence {
            entries,
            completions,
            later,
            projected,
            published_values,
            prior_values,
        },
        &case,
        count,
        duplicate,
        failures,
    )
    }).expect("the pair fixture owns its installed preparation call")
}

#[path = "default_case/mod.rs"]
pub mod default_case;
#[path = "denial_custody/mod.rs"]
pub mod denial_custody;
#[path = "verification/mod.rs"]
pub mod verification;
