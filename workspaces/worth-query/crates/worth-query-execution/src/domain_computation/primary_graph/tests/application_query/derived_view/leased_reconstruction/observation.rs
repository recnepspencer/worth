//! Real admitted pair reads across request backings; projection stays on the owner.
use super::super::*;
use super::{Case, Observation, ScopeInterruption};
mod denial_custody;
mod verification;
use crate::domain_computation::primary_graph::application_contribution::{
    test_execution_authority, test_policy,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    NestedAccountQuery, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};

pub(super) fn observe(
    workers: Option<usize>,
    count: usize,
    duplicate: bool,
    owner_failure: bool,
    failures: &[usize],
) -> Observation {
    observe_case(
        workers,
        count,
        duplicate,
        owner_failure,
        failures,
        Case::default(),
    )
}
pub(super) fn observe_case(
    workers: Option<usize>,
    count: usize,
    duplicate: bool,
    owner_failure: bool,
    failures: &[usize],
    case: Case,
) -> Observation {
    let _run = crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = installed_authorization_world(true);
    if count == 3 {
        super::reconstruction_population::add_third_root(&world);
    }
    if count >= 2 {
        super::reconstruction_population::add_primary_reads(&world, count == 3);
    }
    let query_cancel =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let scope = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        std::time::Instant::now()
            + Duration::from_secs(
                if matches!(case.query_interruption, Some(ScopeInterruption::Deadline)) {
                    5
                } else {
                    60
                },
            ),
        query_cancel.token(),
    );
    let caller_cancel = worth_execution::CancellationSource::new();
    let selected = world.selected_product();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = selected
        .resolve_authenticated_principal(
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
            selected
                .retain_selection()
                .unwrap()
                .admit_application_query(
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
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(16 << 20),
        caller_cancel.token(),
        deadline,
    );
    let lease = workers.map(|workers| {
        test_execution_authority()
            .request_lease(LeaseRequest {
                policy: test_policy(NonZeroUsize::new(workers).unwrap(), serial.memory().limit()),
                cancellation: caller_cancel.token(),
                deadline,
            })
            .unwrap()
    });
    let request = lease.as_ref().map_or_else(
        || ExecutionRequest::serial(&serial),
        ExecutionRequest::leased,
    );
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
        crate::domain_computation::primary_graph::application_query::DispatchWitness::install(
            later,
        );
    if let Some((index, is_deadline)) = case.interrupt {
        let root = canonical_roots[index];
        let earlier = canonical_roots[..index].to_vec();
        use crate::domain_computation::primary_graph::application_query::DispatchInterrupt as Interrupt;
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
        use crate::domain_computation::primary_graph::application_query::DispatchInterrupt as Interrupt;
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
                deadline: scope.deadline(),
            },
        });
    }
    let mut projected = Vec::new();
    let mut prepared = 0;
    let mut run = |failures: &[usize], owner_failure: bool| {
        projected.clear();
        prepared = 0;
        witness.reset();
        let result = request.run(ExecutionWorkCeiling::new(u64::MAX), |_| {
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
                            .retain_selection()
                            .unwrap()
                            .admit_application_query(
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
                            .retain_selection()
                            .unwrap()
                            .admit_application_query(
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
            denial_custody::check(error, name);
            denial_custody::bytes(name)
        } else {
            0
        };
    let available = serial.memory().limit().checked_sub(denial_bytes).unwrap();
    if let Some(lease) = &lease {
        drop(lease.reserve_memory(available).unwrap());
    } else {
        drop(serial.memory().reserve(available).unwrap());
    }
    let retained_values = prior
        .iter()
        .map(|key| snapshot.get(key).unwrap().unwrap().0.clone())
        .collect::<Vec<_>>();
    let retention = view.reconstruction_retention_for_test();
    verification::verify(
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
        verification::ApplicationEvidence {
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
}
