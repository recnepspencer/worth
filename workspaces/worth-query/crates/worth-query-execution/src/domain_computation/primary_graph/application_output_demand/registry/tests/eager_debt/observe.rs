//! Compare reader-visible values and retained debt after every operation.
use super::super::super::refreshed_rejoin::infallible;
use super::*;
pub(super) fn compare(
    world: &World<'_>,
    seed: u64,
    step: usize,
    action: &str,
    divergences: &mut Vec<String>,
    quiescent: bool,
) {
    let state = world
        .registry
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut differences = Vec::new();
    for (id, key) in world.keys.iter().enumerate() {
        let physical = state.records.get(key);
        let observation = infallible(super::super::super::claim_readthrough::observe_row(
            &state.records,
            key,
            &mut OracleWalk::after_effect(),
        ));
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
                let phase = if super::super::super::refreshed_rejoin::superseded(row) {
                    Phase::Superseded
                } else if row.terminal() {
                    Phase::Closed
                } else if row.has_cached_ready() {
                    Phase::Ready
                } else if matches!(row.state, DemandState::Running) {
                    Phase::Running
                } else {
                    Phase::Pending
                };
                if phase != model.phase
                    || row.source_commits.contains(&world.context.commit(1)) != model.source
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
        let visible = super::super::super::post_commit::custody_reader::visible(
            &state,
            &world.context.commit(id),
        );
        if visible != expected.is_some() {
            differences.push(format!("reader custody {id} presence differs"));
        }
        if let (Some(actual), Some(expected)) = (actual.filter(|_| visible), expected) {
            let bound = actual.bound_sources.clone();
            let expected_bound = expected.bound.map(|generation| {
                vec![super::super::super::BoundOutputSource {
                    scope: world.context.scope,
                    identity: world
                        .keys
                        .iter()
                        .find(|key| {
                            key.producer == "oracle-upstream"
                                && key.source.observation_generation() == generation
                        })
                        .unwrap()
                        .source
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
                            source.identity.clone(),
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
        let word = std::mem::size_of::<usize>();
        let pointer = std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        let member_layout =
            std::mem::size_of::<WorthQueryOutputDemandKey>() + 13 * pointer + 16 * word;
        let required = expected_members
            .iter()
            .map(|key| member_layout + key.producer.len())
            .sum::<usize>()
            + world
                .model
                .rows
                .values()
                .map(|r| r.claim_slots * pointer)
                .sum::<usize>();
        let actual_totals = [
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
        ];
        if actual_totals != [required, 0, 0] {
            differences.push(format!(
                "final reserved totals {actual_totals:?} != {:?}",
                [required, 0, 0]
            ));
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

pub(super) fn dangling(world: &World<'_>) -> Option<WorthQueryOutputDemandKey> {
    let state = world
        .registry
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (!state.closed_cleanup.valid_order()).then(|| world.keys[2].clone())
}
