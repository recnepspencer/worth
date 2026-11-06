//! Full-entry eligibility is independent of the noted order.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::refresh_order;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::superseded;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::custody_note::*;
use std::cmp::Ordering;
use worth_runtime_world::facade::CompositeCommitIdentity;

pub fn eligible(state: &DemandRegistryState, commit: &CompositeCommitIdentity) -> bool {
    let entry = &state.source_custody[commit];
    if (*entry.token_count_for_test()) != 0 {
        return false;
    }
    if (*entry.retired_for_test()).is_some() {
        return true;
    }
    let Some(bound) = entry.bound_sources_for_test() else {
        return false;
    };
    let replaced = !bound.is_empty()
        && bound.iter().all(|s| {
            entry
                .source_denial_for_test(s.identity_for_test())
                .is_some()
        });
    if !(*entry.completed_for_test()) && !replaced {
        return false;
    }
    if !bound.iter().all(|s| {
        (*entry.consumed_sources_for_test()).contains(s.identity_for_test())
            || entry
                .source_denial_for_test(s.identity_for_test())
                .is_some()
    }) {
        return false;
    }
    !(*state.records_for_test())
        .iter_for_test()
        .any(|(key, row)| {
            let in_source = bound.iter().any(|s| {
                (*key.source_for_test()).occurrence_coordinate()
                    == (*s.identity_for_test()).occurrence_coordinate()
            });
            if !in_source || !(*row.source_commits_for_test()).contains(commit) {
                return false;
            }
            let newer = (*state.records_for_test())
                .iter_for_test()
                .any(|(other, candidate)| {
                    key.same_occurrence_for_test(other)
                        && refresh_order(other, key) == Some(Ordering::Greater)
                        && !candidate.unpublished_new_key_for_test()
                });
            let awaited = !newer
                && (*state.records_for_test())
                    .iter_for_test()
                    .any(|(other, candidate)| {
                        key.same_occurrence_for_test(other)
                            && refresh_order(other, key) == Some(Ordering::Less)
                            && superseded(candidate)
                            && ((*candidate.interests_for_test()) != 0
                                || (*candidate.framework_required_count_for_test()) != 0)
                    });
            (*row.interests_for_test()) != 0
                || awaited
                || !(*row.performed_obligations_for_test()).is_empty()
                || !(row.has_cached_ready_for_test()
                    || matches!(*row.state_for_test(), DemandState::Failed(_)))
        })
}

pub fn assert_noted(registry: &WorthQueryOutputDemandRegistry, step: &str, seed: u64) {
    let state = (*registry.state_for_test()).lock().unwrap();
    for (commit, entry) in &state.source_custody {
        assert!(
            !eligible(&state, commit) || entry.cleanup_for_test().queued,
            "seed {seed}, {step}: prunable custody lost its note"
        );
    }
}
pub fn eager(registry: &WorthQueryOutputDemandRegistry) {
    registry.drain_closed_cleanup_admitted().unwrap();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    let removed: Vec<_> = state
        .source_custody
        .keys()
        .filter(|c| eligible(&state, c))
        .cloned()
        .collect();
    for commit in removed {
        state.discard_custody_for_test(&commit);
        for mut row in (*state.records_mut_for_test()).values_mut_for_test() {
            (*row.source_commits_mut_for_test()).retain(|c| c != &commit);
        }
    }
}

pub fn value(
    registry: &WorthQueryOutputDemandRegistry,
) -> impl PartialEq + std::fmt::Debug + use<> {
    let state = (*registry.state_for_test()).lock().unwrap();
    let records: Vec<_> = (*state.records_for_test())
        .iter_for_test()
        .map(|(key, row)| {
            (
                key.clone(),
                (*row.interests_for_test()),
                (*row.required_interests_for_test()),
                (*row.framework_required_count_for_test()),
                (*row.source_commits_for_test()).clone(),
                (*row.prerequisites_for_test()).clone(),
                row.has_cached_ready_for_test(),
                (*row.settlements_for_test()).clone(),
                (
                    match row.state_for_test() {
                        DemandState::Admitted => "admitted",
                        DemandState::Scheduling => "scheduling",
                        DemandState::Scheduled => "scheduled",
                        DemandState::Running => "running",
                        DemandState::Output(_) => "output",
                        DemandState::Failed(_) => "failed",
                    },
                    match row.state_for_test() {
                        DemandState::Failed(d) => Some(d.kind()),
                        _ => None,
                    },
                ),
            )
        })
        .collect();
    let custody: Vec<_> = state
        .source_custody
        .iter()
        .map(|(commit, row)| {
            (
                commit.clone(),
                (*row.bound_sources_for_test()).clone(),
                (*row.consumed_sources_for_test()).clone(),
                (*row.retired_sources_for_test()).clone(),
                (*row.retired_for_test()).clone(),
                (*row.completed_for_test()),
                (*row.token_count_for_test()),
                (*row.occurrence_for_test()),
                (*row.root_kind_for_test()).clone(),
            )
        })
        .collect();
    (records, custody, (*state.required_keys_for_test()).clone())
}

#[test]
fn registry_custody_note_full_entry_oracle_covers_held_unhold_steps() {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let ready = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    let scope = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence().0;
    let (_lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let mut visits = [0; 4];
    for seed in 0..16 {
        let receipt = &receipts[(seed % 4) as usize];
        let commit = receipt.committed_product_publication().composite_commit();
        let make = || {
            let result = fixture(receipt, &ready, seed, scope);
            let foreign: Vec<_> = receipts
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != (seed % 4) as usize)
                .take((seed / 4) as usize)
                .map(|(_, r)| r.clone())
                .collect();
            let mut state = (*result.0.state_for_test()).lock().unwrap();
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody_in_scope(&mut state, &foreign, 1 + seed % 3, scope);
            for (key, mut row) in &mut (*state.records_mut_for_test()) {
                if (*key.producer_for_test()).starts_with("custody-unrelated-") {
                    (*row.interests_mut_for_test()) = 0;
                    (*row.state_mut_for_test()) =
                        DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                            WorthQueryOutputDemandDenialKind::NoEffect,
                            "completed neighbor",
                        ));
                }
            }
            drop(state);
            result
        };
        let (registry, owner, dependent, newest) = make();
        let (reference, eager_owner, eager_dependent, _) = make();
        for reg in [&registry, &reference] {
            let mut state = (*reg.state_for_test()).lock().unwrap();
            if seed & 4 != 0 {
                (*state
                    .source_custody
                    .get_mut(commit)
                    .unwrap()
                    .token_count_mut_for_test()) = 1;
            }
            // A second closed, settled dependent requires newest independently
            // of the stale edge being released or transferred in this seed.
            let member = Arc::clone((*state.required_keys_for_test()).get(&newest).unwrap());
            let mut other = record(occurrence(), copy_ready(&ready), 0);
            (*other.prerequisites_mut_for_test()).push(member);

            (*state.records_mut_for_test())
                .insert_for_test(key("other-dependent", seed + 1, 3), other);
            (*(*state.records_mut_for_test())
                .get_mut(&newest)
                .unwrap()
                .framework_required_count_mut_for_test()) = 1;
            // Begin with a published predecessor: this step must actually
            // install Superseded, rather than revisit an already stopped row.
            let predecessor = key_with_identity("upstream", seed + 1, 1, 10);
            retain_owned_key(&mut state, &predecessor);
            (*(*state.records_mut_for_test())
                .get_mut(&predecessor)
                .unwrap()
                .state_mut_for_test()) = copy_ready(&ready);
            assert!(!superseded(&(*state.records_for_test())[&predecessor]));
            supersede_predecessors(&mut state, &newest).unwrap();
            assert!(superseded(&(*state.records_for_test())[&predecessor]));
        }
        assert_noted(&registry, "supersede", seed);
        drop(owner);
        drop(eager_owner);
        eager(&reference);
        assert_noted(&registry, "owner drop", seed);
        (*registry.state_for_test())
            .lock()
            .unwrap()
            .drain_next_foreign_custody_for_test();
        eager(&reference);
        assert_noted(&registry, "admission while held", seed);
        let step = (seed % 4) as usize;
        visits[step] += 1;
        match step {
            0 => {
                drop(dependent);
                drop(eager_dependent);
            }
            1 => {
                for (reg, dependent) in [(&registry, &dependent), (&reference, &eager_dependent)] {
                    let mut state = (*reg.state_for_test()).lock().unwrap();
                    (*(*state.records_mut_for_test())
                        .get_mut(dependent.key_for_test())
                        .unwrap()
                        .state_mut_for_test()) =
                        DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                            WorthQueryOutputDemandDenialKind::NoEffect,
                            "done",
                        ));
                    state.release_record_prerequisites_for_test(dependent.key_for_test());
                }
                drop(dependent);
                drop(eager_dependent);
            }
            2 | 3 => {
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::registry::prerequisite_claims::custody_note_steps::unhold(
                    &dependent,
                    step == 2,
                    &identity,
                );
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::registry::prerequisite_claims::custody_note_steps::unhold(
                    &eager_dependent,
                    step == 2,
                    &identity,
                );
                drop(dependent);
                drop(eager_dependent);
            }
            _ => unreachable!(),
        }
        eager(&reference);
        assert_noted(
            &registry,
            [
                "claim transfer/dependent drop",
                "obligations release",
                "cancel",
                "publish",
            ][step],
            seed,
        );
        if seed & 4 != 0 {
            registry.release_prepared_token_for_test(commit);
            reference.release_prepared_token_for_test(commit);
            eager(&reference);
            assert_noted(&registry, "token drop", seed);
        }
        registry.drain_closed_cleanup_admitted().unwrap();
        assert_noted(&registry, "paid claim transfer", seed);
        let remaining = (*registry.state_for_test())
            .lock()
            .unwrap()
            .custody_cleanup_for_test()
            .len;
        for _ in 0..remaining {
            (*registry.state_for_test())
                .lock()
                .unwrap()
                .drain_next_foreign_custody_for_test();
            assert_noted(&registry, "quiescent cleanup", seed);
        }
        eager(&reference);
        assert!(!(*registry.state_for_test())
            .lock()
            .unwrap()
            .pending_custody_prune_for_test());
        assert_eq!(value(&registry), value(&reference), "seed {seed}");
    }
    assert!(visits.into_iter().all(|count| count != 0));
}
