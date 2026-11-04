//! Compare reader-visible values and retained debt after every operation.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::*;
pub fn compare(
    world: &World<'_>,
    seed: u64,
    step: usize,
    action: &str,
    divergences: &mut Vec<String>,
    quiescent: bool,
) {
    let state = (*world
        .registry
        .state_for_test())
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut differences = Vec::new();
    for (id, key) in world.keys.iter().enumerate() {
        let physical = state.records.get(key);
        let observation = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::claim_readthrough::visibility::observe_row(&state.records, key);
        let (awaited, required, held) =
            (observation.awaited, observation.required, observation.held);
        if (awaited, required, held)
            != (
                world.model.awaited(id),
                world.model.required(id),
                world.model.held(id),
            )
        {
            differences.push(format!(
                "reader row {id}: {:?} != {:?}",
                (awaited, required, held),
                (
                    world.model.awaited(id),
                    world.model.required(id),
                    world.model.held(id)
                )
            ));
        }
        if !world.model.rows.contains_key(&id)
            && physical.is_some_and(|r| !r.closed_cleanup.queued() && !r.pending_cleanup_queued)
        {
            differences.push(format!("removed row {id} unnoted"));
        }
        if quiescent {
            let actual = physical.map(|r| {
                (
                    r.interests,
                    r.framework_required_count,
                    r.prerequisites
                        .iter()
                        .map(|k| {
                            world
                                .keys
                                .iter()
                                .position(|known| known == k.as_ref())
                                .expect("known claim")
                        })
                        .collect::<Vec<_>>(),
                )
            });
            let expected = world
                .model
                .rows
                .get(&id)
                .map(|r| (r.owners, world.model.claimed(id), r.claims.clone()));
            if actual != expected {
                differences.push(format!("final row {id}: {actual:?} != {expected:?}"));
            }
            if let (Some(row), Some(model)) = (physical, world.model.rows.get(&id)) {
                let phase = if worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::superseded(row) {
                    Phase::Superseded
                } else if row.terminal_for_test() {
                    Phase::Closed
                } else if row.has_cached_ready_for_test() {
                    Phase::Ready
                } else if matches!((*row.state_for_test()), DemandState::Running) {
                    Phase::Running
                } else {
                    Phase::Pending
                };
                if phase != model.phase
                    || (*row.source_commits_for_test()).contains(&world.context.commit(1)) != model.source
                {
                    differences.push(format!("final row {id} phase/source differs"));
                }
            }
        }
    }
    for id in 0..2 {
        let actual = state.source_custody.get(&world.context.commit(id));
        let expected = world.model.custody.get(&id);
        if expected.is_none() && actual.is_some_and(|c| !c.cleanup.queued) {
            differences.push(format!("removed custody {id} unnoted"));
        }
        let visible = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::custody_reader::visible(
            &state,
            &world.context.commit(id),
        );
        if visible != expected.is_some() {
            differences.push(format!("reader custody {id} presence differs"));
        }
        if let (Some(actual), Some(expected)) = (actual.filter(|_| visible), expected) {
            let bound = actual.bound_sources.clone();
            let expected_bound = expected.bound.map(|generation| {
                vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource {
                    scope: world.context.scope,
                    identity: (*world
                        .keys
                        .iter()
                        .find(|key| {
                            (*key.producer_for_test()) == "oracle-upstream"
                                && (*key.source_for_test()).observation_generation() == generation
                        })
                        .unwrap()
                        .source_for_test())
                        .clone(),
                }]
            });
            let denied = actual
                .retired_sources
                .iter()
                .map(|(source, cause)| (source.clone(), cause.kind()))
                .collect::<Vec<_>>();
            let expected_denied = if expected.denied {
                expected_bound
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|source| {
                        (
                            (*source.identity_for_test()).clone(),
                            WorthQueryOutputDemandDenialKind::Superseded,
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            // Binding identity and denial are values, never queue-presence proxies.
            if bound != expected_bound || denied != expected_denied {
                differences.push(format!(
                    "binding {id}: bound value matches={}, denial {:?} != {:?}",
                    bound == expected_bound,
                    denied.iter().map(|(_, kind)| kind).collect::<Vec<_>>(),
                    expected_denied
                        .iter()
                        .map(|(_, kind)| kind)
                        .collect::<Vec<_>>()
                ));
            }
        }
        if quiescent && actual.is_some() != expected.is_some() {
            differences.push(format!("final custody {id} differs"));
        }
    }
    if quiescent {
        let expected_members: Vec<_> = world
            .keys
            .iter()
            .enumerate()
            .filter(|(id, _)| world.model.required(*id) || world.model.awaited(*id))
            .map(|(_, key)| key)
            .collect();
        let actual_members: Vec<_> = world
            .keys
            .iter()
            .filter(|key| state.required_keys.contains(*key))
            .collect();
        if actual_members != expected_members {
            differences.push("final member set differs".into());
        }
    }
    if quiescent && (state.closed_cleanup.len() != 0 || state.custody_cleanup.len != 0) {
        differences.push("final note orders nonempty".into());
    }
    if !differences.is_empty() {
        divergences.push(format!(
            "seed {seed}, step {step}, {action}: {}",
            differences.join("; ")
        ));
    }
}

pub fn dangling(world: &World<'_>) -> Option<WorthQueryOutputDemandKey> {
    let state = (*world
        .registry
        .state_for_test())
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (!state.closed_cleanup.valid_order()).then(|| world.keys[2].clone())
}
