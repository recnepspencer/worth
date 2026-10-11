//! Full-entry eligibility is independent of the noted order.
use super::super::super::refreshed_rejoin::{refresh_order, superseded};
use super::*;
use std::cmp::Ordering;
use worth_runtime_world::facade::CompositeCommitIdentity;

fn eligible(state: &DemandRegistryState, commit: &CompositeCommitIdentity) -> bool {
    let entry = &state.source_custody[commit];
    if entry.token_count != 0 {
        return false;
    }
    if entry.retired.is_some() {
        return true;
    }
    let Some(bound) = &entry.bound_sources else {
        return false;
    };
    let replaced = !bound.is_empty()
        && bound
            .iter()
            .all(|s| entry.source_denial(&s.identity).is_some());
    if !entry.completed && !replaced {
        return false;
    }
    if !bound.iter().all(|s| {
        entry.consumed_sources.contains(&s.identity) || entry.source_denial(&s.identity).is_some()
    }) {
        return false;
    }
    !state.records.iter().any(|(key, row)| {
        let in_source = bound
            .iter()
            .any(|s| key.source.occurrence_coordinate() == s.identity.occurrence_coordinate());
        if !in_source || !row.source_commits.contains(commit) {
            return false;
        }
        let newer = state.records.iter().any(|(other, candidate)| {
            key.same_occurrence(other)
                && refresh_order(other, key) == Some(Ordering::Greater)
                && !candidate.unpublished_new_key()
        });
        let awaited = !newer
            && state.records.iter().any(|(other, candidate)| {
                key.same_occurrence(other)
                    && refresh_order(other, key) == Some(Ordering::Less)
                    && superseded(candidate)
                    && (candidate.interests != 0 || candidate.framework_required_count != 0)
            });
        row.interests != 0
            || awaited
            || !row.performed_obligations.is_empty()
            || !(row.has_cached_ready() || matches!(row.state, DemandState::Failed(_)))
    })
}

fn assert_noted(registry: &WorthQueryOutputDemandRegistry, step: &str, seed: u64) {
    let state = registry.state.lock().unwrap();
    for (commit, entry) in &state.source_custody {
        assert!(
            !eligible(&state, commit) || entry.cleanup.queued,
            "seed {seed}, {step}: prunable custody lost its note"
        );
    }
}
fn eager(registry: &WorthQueryOutputDemandRegistry) {
    registry
        .drain_closed_cleanup_admitted(&mut record_admission())
        .unwrap();
    let mut state = registry.state.lock().unwrap();
    let removed: Vec<_> = state
        .source_custody
        .keys()
        .filter(|c| eligible(&state, c))
        .cloned()
        .collect();
    for commit in removed {
        state.discard_custody(&commit);
        for mut row in state.records.values_mut() {
            row.source_commits.retain(|c| c != &commit);
        }
    }
}

pub(in super::super::super) fn value(
    registry: &WorthQueryOutputDemandRegistry,
) -> impl PartialEq + std::fmt::Debug {
    let state = registry.state.lock().unwrap();
    let records: Vec<_> = state
        .records
        .iter()
        .map(|(key, row)| {
            (
                key.clone(),
                row.interests,
                row.required_interests,
                row.framework_required_count,
                row.source_commits.clone(),
                row.prerequisites.clone(),
                row.has_cached_ready(),
                row.settlements.clone(),
                (
                    match &row.state {
                        DemandState::Admitted => "admitted",
                        DemandState::Scheduling => "scheduling",
                        DemandState::Scheduled => "scheduled",
                        DemandState::Running => "running",
                        DemandState::Output(_) => "output",
                        DemandState::Failed(_) => "failed",
                    },
                    match &row.state {
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
                row.bound_sources.clone(),
                row.consumed_sources.clone(),
                row.retired_sources.clone(),
                row.retired.clone(),
                row.completed,
                row.token_count,
                row.occurrence,
                row.root_kind.clone(),
            )
        })
        .collect();
    (
        records,
        custody,
        state.required_keys.clone(),
        [
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
        ],
    )
}

#[test]
fn registry_custody_note_full_entry_oracle_covers_held_unhold_steps() {
    let receipts = walk_bounds::receipt_batch();
    let ready = closed_retirement::ready();
    let scope = source_custody::real_scope_and_occurrence().0;
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
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
            let mut state = result.0.state.lock().unwrap();
            walk_bounds::add_custody_in_scope(&mut state, &foreign, 1 + seed % 3, scope);
            for (key, mut row) in &mut state.records {
                if key.producer.starts_with("custody-unrelated-") {
                    row.interests = 0;
                    row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
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
            let mut state = reg.state.lock().unwrap();
            if seed & 4 != 0 {
                state.source_custody.get_mut(commit).unwrap().token_count = 1;
            }
            // A second closed, settled dependent requires newest independently
            // of the stale edge being released or transferred in this seed.
            let member = Arc::clone(state.required_keys.get(&newest).unwrap());
            let mut other = record(occurrence(), copy_ready(&ready), 0);
            other.prerequisites.push(member);
            state.required_reserved_bytes += other.prerequisites.capacity()
                * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state
                .records
                .insert(key("other-dependent", seed + 1, 3), other);
            state
                .records
                .get_mut(&newest)
                .unwrap()
                .framework_required_count = 1;
            // Begin with a published predecessor: this step must actually
            // install Superseded, rather than revisit an already stopped row.
            let predecessor = key_with_identity("upstream", seed + 1, 1, 10);
            retain_owned_key(&mut state, &predecessor);
            state.records.get_mut(&predecessor).unwrap().state = copy_ready(&ready);
            assert!(!superseded(&state.records[&predecessor]));
            supersede_predecessors(&mut state, &newest, &mut record_admission()).unwrap();
            assert!(superseded(&state.records[&predecessor]));
        }
        assert_noted(&registry, "supersede", seed);
        drop(owner);
        drop(eager_owner);
        eager(&reference);
        assert_noted(&registry, "owner drop", seed);
        registry
            .state
            .lock()
            .unwrap()
            .drain_foreign_custody_quota(&mut record_admission());
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
                    let mut state = reg.state.lock().unwrap();
                    state.records.get_mut(&dependent.key).unwrap().state =
                        DemandState::Failed(WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::NoEffect,
                            "done",
                        ));
                    super::super::super::refreshed_rejoin::infallible(
                        state.release_record_prerequisites(
                            &dependent.key,
                            &mut OracleWalk::after_effect(),
                        ),
                    );
                }
                drop(dependent);
                drop(eager_dependent);
            }
            2 | 3 => {
                super::super::super::prerequisite_claims::custody_note_steps::unhold(
                    &dependent,
                    step == 2,
                    &identity,
                );
                super::super::super::prerequisite_claims::custody_note_steps::unhold(
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
            registry.release_prepared_token(commit);
            reference.release_prepared_token(commit);
            eager(&reference);
            assert_noted(&registry, "token drop", seed);
        }
        registry
            .drain_closed_cleanup_admitted(&mut record_admission())
            .unwrap();
        assert_noted(&registry, "paid claim transfer", seed);
        let remaining = registry.state.lock().unwrap().custody_cleanup.len;
        for _ in 0..remaining
            .div_ceil(super::super::super::cleanup::custody::FOREIGN_CUSTODY_DRAIN_QUOTA)
        {
            registry
                .state
                .lock()
                .unwrap()
                .drain_foreign_custody_quota(&mut record_admission());
            assert_noted(&registry, "quiescent admission", seed);
        }
        eager(&reference);
        assert!(!registry.state.lock().unwrap().pending_custody_prune());
        assert_eq!(value(&registry), value(&reference), "seed {seed}");
    }
    assert!(visits.into_iter().all(|count| count != 0));
}
