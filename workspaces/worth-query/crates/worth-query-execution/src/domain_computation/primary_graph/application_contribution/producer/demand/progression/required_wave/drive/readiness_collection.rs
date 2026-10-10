//! Prepare only exact rows whose consumed predecessors are Current on this basis.
use super::*;
use super::super::member_key::RequiredMemberKey;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
use super::super::super::super::required_continuations::PreparedContinuationWave;

pub(super) struct MemberStop {
    pub index: usize,
    pub stop: WorthQueryOutputDemandDenial,
}
pub(super) struct PreparedRequiredWave<'owner, Schema: ApplicationSchema> {
    pub members: Vec<(
        RequiredMemberKey,
        OwnerPlan<'owner, RequiredFreshProgress<Schema>, MemberStop>,
    )>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect<'owner, 'custody, Schema, Family>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,
    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    wave: &'owner RequiredWaveSelection<'owner, Schema>,
    frontier: &mut RequiredFrontier,
    queue: &mut QueuedRequiredRoots,
    resolved: &mut ResolvedOnWave,
    caller_storage: &mut PreparedContinuationWave<'custody, Schema>,
    queue_storage: &mut PreparedContinuationWave<'custody, Schema>,
    caller_installed: &InstalledProducerProvider<Schema>,
    caller_contacts: usize,
    caller_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind,
    authority: &WorthQueryProducerCommitAuthority,
    performed: &'owner std::cell::RefCell<super::super::performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedRequiredWave<'owner, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let mut plans = Vec::new();
    loop {
        let mut index = 0;
        while index < frontier.members.len() {
            if !matches!(frontier.members[index].state, MemberState::Inspect) {
                index += 1;
                continue;
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
                    .declared_required_predecessors(frontier.members[index].key(), admission)?;
                let mut pending = None;
                for (upstream, member) in frontier.members.iter().enumerate() {
                    if upstream != index
                        && member.state.is_unfinished_predecessor()
                        && declared.contains_occurrence(member.key(), admission)?
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
            let queued = frontier.members[index].queued;
            let successor = frontier.members[index].resolved_successor;
            if let MemberInput::Queued { claim, ready } = &mut frontier.members[index].ready {
                let queued = queued.expect("a claim has queue custody");
                if successor {
                    queue.select_successor_cause(queued, claim, runtime, wave, admission)?;
                } else {
                    let Some(selected) =
                        queue.inspect_claim(queued, claim, runtime, wave, admission)?
                    else {
                        frontier.members[index].state = MemberState::UnselectedClaim;
                        frontier.members[index].queued = None;
                        index += 1;
                        continue;
                    };
                    *ready = Some(selected);
                }
            }
            let inspection = performed.borrow().inspection();
            admission
                .charge_external_work(9)
                .map_err(|_| work_denial())?;
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
            let installation_work = storage.reserve_one(&runtime.output_demands, admission)?;
            let mut installation = admission
                .reserve_external_work(installation_work)
                .map_err(admission_denial)?;
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
                installation.admission(),
            );
            installation
                .settle(if matches!(&result, Ok(RequiredWaveStep::Fresh(_))) {
                    installation_work
                } else {
                    0
                })
                .map_err(admission_denial)?;
            match result {
                Ok(RequiredWaveStep::Current(settlement)) => {
                    let storage = if caller {
                        &mut *caller_storage
                    } else {
                        &mut *queue_storage
                    };
                    storage.release_unused();
                    runtime.output_demands.clear_required_stop(member.key());
                    let refresh = member.resolved_successor;
                    if !caller && queued.is_some() && !required_by_member {
                        // A reached Current row is not a newly resolved read.
                        // Only the actual refreshed successor can rebind an old
                        // consumed edge for the following queue component.
                        resolved.clear();
                    }
                    if queued.is_none() || refresh || required_by_member {
                        resolved.push(
                            member.ready.selected().retain(admission)?,
                            Arc::clone(&settlement),
                            admission,
                        )?;
                    }
                    frontier.members[index].state =
                        MemberState::Current((index == 0).then_some(settlement));
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
                        admission,
                    )? {
                        frontier.members[index].state = MemberState::Pending;
                        index += 1;
                        continue;
                    }
                    frontier.members[index].dependency_schedule =
                        DependencySchedule::PendingConsumed;
                    let prerequisites = runtime.output_demands.declared_required_readmissions(
                        frontier.members[index].ready.selected(),
                        admission,
                    )?;
                    for prerequisite in prerequisites {
                        // Declared membership schedules inspection; only the
                        // actual Upstream edge can ask the caller for work.
                        frontier.insert(prerequisite, caller, None, admission)?;
                    }
                    match frontier.require_upstream(ready, caller, admission)? {
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
                        admission,
                    )?;
                    let slot_bytes = std::mem::size_of::<(
                        RequiredMemberKey,
                        OwnerPlan<'owner, RequiredFreshProgress<Schema>, MemberStop>,
                    )>();
                    if plans.len() == plans.capacity() {
                        let next = plans
                            .capacity()
                            .checked_mul(2)
                            .ok_or_else(capacity_denial)?
                            .max(1);
                        let bytes = next.checked_mul(slot_bytes).ok_or_else(capacity_denial)?;
                        admission
                            .admit_read_scratch(
                                u64::try_from(bytes).map_err(|_| capacity_denial())?,
                            )
                            .map_err(admission_denial)?;
                        admission
                            .charge_external_work(
                                u64::try_from(
                                    plans
                                        .len()
                                        .checked_mul(slot_bytes)
                                        .ok_or_else(work_denial)?,
                                )
                                .map_err(|_| work_denial())?,
                            )
                            .map_err(|_| work_denial())?;
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
                        plan.map_result(move |result, _| {
                            result.map_err(|stop| MemberStop { index, stop })
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
    // Comparisons are charged before inspecting semantic producer/source keys.
    // Once admission refuses, no further key comparison is made.
    let mut refused = false;
    plans.sort_unstable_by(|(left, _), (right, _)| {
        if refused {
            return std::cmp::Ordering::Equal;
        }
        if left
            .comparison_work(right)
            .and_then(|work| {
                admission
                    .charge_external_work(work)
                    .map_err(|_| work_denial())
            })
            .is_err()
        {
            refused = true;
            return std::cmp::Ordering::Equal;
        }
        left.cmp(right)
    });
    if refused {
        return Err(work_denial());
    }
    Ok(PreparedRequiredWave { members: plans })
}
