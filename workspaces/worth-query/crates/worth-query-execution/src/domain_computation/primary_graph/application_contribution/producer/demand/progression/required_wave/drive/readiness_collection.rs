//! Prepare only exact rows whose consumed predecessors are Current on this basis.
use super::*;
use super::super::member_key::RequiredMemberKey;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
use super::super::super::super::required_continuations::PreparedContinuationWave;

pub(super) use super::member_stop::MemberStop;
pub(super) struct PreparedRequiredWave<'owner, Schema: ApplicationSchema> {
    pub members: Vec<(
        RequiredMemberKey,
        OwnerPlan<'owner, RequiredFreshProgress<Schema>, MemberStop>,
    )>,
}

pub(super) enum CollectedRequiredWave<'owner, 'custody, Schema: ApplicationSchema> {
    Frozen(super::frozen_collection::FrozenRequiredWave<'custody, Schema>),
    Live(PreparedRequiredWave<'owner, Schema>),
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect<'owner, 'custody, 'collection, Schema, Family>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,
    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    wave: &'owner RequiredWaveSelection<'owner, Schema>,
    frontier: &mut RequiredFrontier,
    queue: &mut QueuedRequiredRoots,
    resolved: &mut ResolvedOnWave,
    caller_storage: &'collection mut PreparedContinuationWave<'custody, Schema>,
    queue_storage: &mut PreparedContinuationWave<'custody, Schema>,
    caller_installed: &InstalledProducerProvider<Schema>,
    caller_contacts: usize,
    caller_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind,
    authority: &WorthQueryProducerCommitAuthority,
    performed: &'owner std::cell::RefCell<super::super::performed::PerformedMembers>,
) -> Result<CollectedRequiredWave<'owner, 'collection, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    if wave.is_frozen() {
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "advancement phase",
                )
            })?;
        return super::frozen_collection::collect(
            &execution,
            runtime,
            principal,
            request,
            wave,
            frontier,
            caller_storage,
            caller_installed,
            caller_contacts,
            authority,
            performed,
        )
        .map(CollectedRequiredWave::Frozen);
    }
    let mut plans = Vec::new();
    'collect: loop {
        let mut index = 0;
        while index < frontier.members.len() {
            if !matches!(frontier.members[index].state, MemberState::Inspect) {
                index += 1;
                continue;
            }
            // A root decides the caller or requested return before publication.
            // If its upstream woke it while siblings prepared, run that wave
            // first; certify the root on the next pass with no plans held.
            // The declared-predecessor gate subsumes this defense in every
            // topology tested; the guard also forbids future prepared-root certification.
            if index == 0 && !plans.is_empty() {
                break 'collect;
            }
            // Queued work and a caller waiting on a genuine PendingExact edge
            // must leave all owed upstreams Ready before preparing. A caller
            // with changed own facts follows its ordinary native selection;
            // historical consumed edges cannot override that owner's refusal.
            if !frontier.members[index].caller
                || matches!(
                    frontier.members[index].dependency_schedule,
                    DependencySchedule::PendingConsumed,
                )
            {
                let declared = runtime
                    .output_demands
                    .declared_required_predecessors(frontier.members[index].key())?;
                let mut pending = None;
                for (upstream, member) in frontier.members.iter().enumerate() {
                    if upstream != index
                        && member.state.is_unfinished_predecessor()
                        && declared.contains_occurrence(member.key())?
                    {
                        pending = Some(upstream);
                        break;
                    }
                }
                if let Some(upstream) = pending {
                    frontier.wait_for_declaration(index, upstream);
                    index += 1;
                    continue;
                }
            }
            if super::retained_refresh::defer(
                &mut frontier.members[index],
                caller_storage,
                queue_storage,
            )? {
                index += 1;
                continue;
            }
            let queued = frontier.members[index].queued;
            let successor = frontier.members[index].resolved_successor;
            if let Some(claim) = frontier.members[index].ready.claim_mut() {
                let queued = queued.expect("a claim has queue custody");
                if successor {
                    queue.select_successor_cause(queued, claim, runtime, wave)?;
                } else {
                    let Some(selected) = queue.inspect_claim(queued, claim, runtime, wave)? else {
                        frontier.members[index].state = MemberState::UnselectedClaim;
                        frontier.members[index].queued = None;
                        index += 1;
                        continue;
                    };
                    frontier.members[index]
                        .ready
                        .bind_inspected_native(selected);
                }
            }
            let inspection = performed.borrow().inspection();

            let required_by_member = frontier.members.iter().any(|member| {
                matches!(member.state, MemberState::Waiting(upstream) if upstream == index)
            });
            let member = &frontier.members[index];
            let caller = member.caller;
            let queued = member.queued;
            if caller && wave.target == RequiredWaveTarget::Caller {
                member.ready.selected().completion().advanced_in(authority);
            }
            let contacts = if index == 0 && wave.target == RequiredWaveTarget::Caller {
                caller_contacts
            } else {
                member.contacts
            };
            let storage = if caller {
                &mut *caller_storage
            } else {
                &mut *queue_storage
            };
            storage.reserve_one(&runtime.output_demands)?;
            let view = resolved.view_two(caller_storage.entries(), queue_storage.entries());
            let permission = super::super::super::refresh::permit_refresh(
                if index == 0 && wave.target == RequiredWaveTarget::Caller {
                    caller_kind
                } else {
                    crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required
                },
                Family::IDENTITY,
            );
            let result = certify_required_ready(
                phase,
                runtime,
                principal,
                request,
                wave,
                member.ready.selected(),
                (index == 0
                    && wave.target == RequiredWaveTarget::Caller
                    && !member.caller_successor)
                    .then_some(caller_installed),
                view.as_ref(),
                contacts,
                permission,
                performed,
            );
            match result {
                Ok(RequiredWaveStep::Current(proof)) => {
                    let storage = if caller {
                        &mut *caller_storage
                    } else {
                        &mut *queue_storage
                    };
                    storage.release_unused();
                    runtime.output_demands.clear_required_stop(member.key());
                    if index == 0 {
                        frontier.members[index].state = MemberState::Current(Some(proof));
                        return Ok(CollectedRequiredWave::Live(PreparedRequiredWave {
                            members: Vec::new(),
                        }));
                    }
                    let refresh = member.resolved_successor;
                    if !caller && queued.is_some() && !required_by_member {
                        // A reached Current row is not a newly resolved read.
                        // Only the actual refreshed successor can rebind an old
                        // consumed edge for the following queue component.
                        resolved.clear();
                    }
                    if queued.is_none() || refresh || required_by_member {
                        resolved
                            .push(member.ready.selected().retain()?, Arc::clone(proof.value()))?;
                    }
                    if let Some(queued) = queued {
                        queue.acknowledge_root(queued, proof)?;
                        frontier.members[index].ready.finish_claim();
                        frontier.members[index].queued = None;
                    }
                    frontier.members[index].state = MemberState::Current(None);
                }
                Ok(RequiredWaveStep::Upstream(ready)) => {
                    let storage = if caller {
                        &mut *caller_storage
                    } else {
                        &mut *queue_storage
                    };
                    storage.release_unused();
                    // A native Upstream asks for work. Preserve the same
                    // self/caller/resolved cycle decision as a requested edge,
                    // before declarations can insert or reopen any row.
                    if super::super::cycles::upstream_cycles(
                        &ready,
                        frontier.members[index].ready.selected(),
                        &wave.anchor_ready,
                        resolved,
                    )? {
                        frontier.members[index].state = MemberState::Pending;
                        index += 1;
                        continue;
                    }
                    frontier.members[index].dependency_schedule =
                        DependencySchedule::PendingConsumed;
                    let prerequisites = runtime
                        .output_demands
                        .declared_required_readmissions(frontier.members[index].ready.selected())?;
                    for prerequisite in prerequisites {
                        // Declared membership schedules inspection; only the
                        // actual Upstream edge can ask the caller for work.
                        frontier.insert(prerequisite, caller, None)?;
                    }
                    match frontier.require_upstream(ready, caller)? {
                        RequiredUpstream::Member(upstream) => {
                            frontier.wait_for(index, upstream);
                        }
                        RequiredUpstream::HeldCaller => {
                            frontier.members[index].state = MemberState::Pending;
                        }
                    }
                }
                Ok(RequiredWaveStep::Held(head)) => {
                    let storage = if caller {
                        &mut *caller_storage
                    } else {
                        &mut *queue_storage
                    };
                    storage.release_unused();
                    frontier.members[index].state = MemberState::Held(head)
                }
                Ok(RequiredWaveStep::Pending) => {
                    let storage = if caller {
                        &mut *caller_storage
                    } else {
                        &mut *queue_storage
                    };
                    storage.release_unused();
                    frontier.members[index].state = MemberState::Pending
                }
                plan => {
                    let plan = match plan {
                        Ok(RequiredWaveStep::Fresh(plan)) => plan,
                        Err(stop) => OwnerPlan::immediate(Err(stop)),
                        Ok(
                            RequiredWaveStep::Current(_)
                            | RequiredWaveStep::Upstream(_)
                            | RequiredWaveStep::Held(_)
                            | RequiredWaveStep::Pending,
                        ) => unreachable!(),
                    };
                    let key = RequiredMemberKey::retain(
                        index,
                        frontier.members[index].ready.selected(),
                        inspection,
                    )?;

                    if plans.len() == plans.capacity() {
                        let next = plans
                            .capacity()
                            .checked_mul(2)
                            .ok_or_else(capacity_denial)?
                            .max(1);

                        let mut replacement = Vec::new();
                        replacement
                            .try_reserve_exact(next)
                            .map_err(|_| capacity_denial())?;
                        if replacement.capacity() != next {
                            return Err(capacity_denial());
                        }
                        replacement.append(&mut plans);
                        plans = replacement;
                    }
                    plans.push((
                        key,
                        plan.map_result(move |result| {
                            result.map_err(|stop| MemberStop::member(index, stop))
                        }),
                    ));
                    frontier.members[index].state = MemberState::Prepared;
                }
            }
            index += 1;
        }
        let awakened = frontier.awaken();
        let readmitted = frontier
            .members
            .iter()
            .any(|member| matches!(member.state, MemberState::Inspect));
        if !awakened && !readmitted {
            break;
        }
    }
    plans.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    Ok(CollectedRequiredWave::Live(PreparedRequiredWave {
        members: plans,
    }))
}
