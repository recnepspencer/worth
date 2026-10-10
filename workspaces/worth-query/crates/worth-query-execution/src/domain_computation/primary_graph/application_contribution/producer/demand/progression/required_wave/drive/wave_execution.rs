//! One wave: owner preparation, owned compute dispatch, consuming ordered apply.
use super::*;
use super::readiness_collection::{self, MemberStop};
use super::super::selection::committed_ready;
use super::super::super::super::RequiredFreshOutcome;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::canonical_apply::CanonicalApplyCursor;

pub(super) struct WaveProgress {
    pub moved: bool,
    pub prepared: bool,
    pub stopped: Option<MemberStop>,
    pub queued_stops: Vec<MemberStop>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run<'owner, Schema, Family>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,
    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    authority: &WorthQueryProducerCommitAuthority,
    wave: &'owner RequiredWaveSelection<'owner, Schema>,
    frontier: &mut RequiredFrontier,
    resolved: &mut ResolvedOnWave,
    queue_custody: &mut RequiredContinuations<Schema>,
    queue: &mut QueuedRequiredRoots,
    performed: &'owner std::cell::RefCell<super::super::performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<WaveProgress, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let mut caller_storage = demand.required_continuations.prepare_wave();
    let mut queue_storage = queue_custody.prepare_wave();
    let prepared = readiness_collection::collect::<Schema, Family>(
        phase,
        runtime,
        principal,
        request,
        wave,
        frontier,
        queue,
        resolved,
        &mut caller_storage,
        &mut queue_storage,
        &demand.installed_entry,
        demand.producer_contacts_in_this_demand,
        demand.admission_kind,
        authority,
        performed,
        admission,
    )?;
    if prepared.members.is_empty() {
        return Ok(WaveProgress {
            moved: false,
            prepared: false,
            stopped: None,
            queued_stops: Vec::new(),
        });
    }
    let (keys, plans): (Vec<_>, Vec<_>) = prepared.members.into_iter().unzip();
    for pair in keys.windows(2) {
        admission
            .charge_external_work(pair[0].comparison_work(&pair[1])?)
            .map_err(|_| work_denial())?;
    }
    let keys = worth_proof::CanonicalUniqueVec::try_from_sorted_unique(keys)
        .map_err(|_| foreign_denial())?;
    let execution = phase
        .execution_for(&runtime.product_runtime)
        .map_err(|_| foreign_denial())?;
    let admitted =
        runtime
            .output_demands
            .admit_prepared_wave(&wave.positioned, keys, plans, admission)?;
    let cursor = CanonicalApplyCursor::compute_admitted(admitted, execution);
    let mut moved = false;
    let mut queued_stops = Vec::new();
    let result = cursor.apply(
        admission,
        |key, outcome, admission| {
            let mut installed = false;
            let applied = (|| {
                let mut progress = outcome?;
                let index = key.index;
                let member = &mut frontier.members[index];
                let storage = if member.caller {
                    &mut caller_storage
                } else {
                    &mut queue_storage
                };
                if index == 0 && wave.target == RequiredWaveTarget::Caller {
                    progress.attribute_to_caller(&mut demand.producer_contacts_in_this_demand);
                }
                member.contacts = progress.producer_contacts();
                let outcome = storage.install_next(progress);
                installed = true;
                match outcome {
                    RequiredFreshOutcome::Refused(stop) => {
                        return Err(MemberStop::member(index, stop))
                    }
                    RequiredFreshOutcome::Advanced => {}
                }
                // Each slot commits, publishes and advances its actual checkpoint
                // before the consuming cursor can yield the next slot.
                loop {
                    let successor = storage.last_mut();
                    let stage = match super::successor_checkpoint::inspect(admission, |admission| {
                        runtime
                            .output_demands
                            .interest_ready_readmission(successor.interest(), admission)
                    }) {
                        Ok(super::successor_checkpoint::CheckpointTurn::Current(ready)) => {
                            runtime
                                .output_demands
                                .clear_required_stop(successor.interest().key());
                            moved |= committed_ready(&ready, admission)
                                .map_err(|stop| MemberStop::member(index, stop))?;
                            member.ready.rebind(ready);
                            member.resolved_successor = true;
                            member.caller_successor |= index == 0;
                            member.state = MemberState::Inspect;
                            break;
                        }
                        Err(stop) => {
                            return Err(MemberStop::checkpoint(
                                index,
                                &runtime.output_demands,
                                successor.interest().key(),
                                stop,
                            ))
                        }
                        Ok(super::successor_checkpoint::CheckpointTurn::Awaiting(stage)) => stage,
                    };
                    match stage.advance(admission, |admission| {
                        storage
                            .last_mut()
                            .advance_checkpoint(phase, runtime, request, admission)
                    }) {
                        Ok(true) => continue,
                        Ok(false) => {
                            member.state = MemberState::Pending;
                            // A deferred delivery leaves the selection on its old head.
                            // Only a committed Ready can move the selected basis.
                            break;
                        }
                        Err(stop) => {
                            return Err(MemberStop::checkpoint(
                                index,
                                &runtime.output_demands,
                                storage.last_mut().interest().key(),
                                stop,
                            ))
                        }
                    }
                }
                Ok(())
            })();
            match applied {
                Err(failure)
                    if failure.stop.kind()
                        == WorthQueryOutputDemandDenialKind::PublicationStale =>
                {
                    // Failed completion custody is discarded on both paths.
                    // The caller returns its refusal; only another publication
                    // since inspection can lend a queued member another turn.
                    if installed {
                        if frontier.members[failure.index].caller {
                            caller_storage.discard_last();
                        } else {
                            queue_storage.discard_last();
                        }
                    }
                    if frontier.members[failure.index].caller {
                        return Err(failure);
                    }
                    let credit = performed
                        .borrow()
                        .reinspection(
                            frontier.members[failure.index].key(),
                            key.inspection,
                            admission,
                        )
                        .map_err(|stop| MemberStop::member(failure.index, stop))?;
                    if credit.is_some() {
                        if let Some(slot) = frontier.members[failure.index].queued {
                            queue.discard_selection(slot);
                        }
                    }
                    if frontier
                        .queued_member(failure.index)
                        .expect("only a queued member can retry")
                        .refused(credit)
                    {
                        moved = true;
                    } else {
                        queued_stops.push(failure);
                    }
                    Ok(())
                }
                Err(failure) if !frontier.members[failure.index].caller => {
                    frontier.members[failure.index].state = MemberState::Pending;
                    queued_stops.push(failure);
                    Ok(())
                }
                result => result,
            }
        },
        |key, denial| {
            MemberStop::member(
                key.index,
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(denial),
                    format!("required member compute refused: {denial}"),
                ),
            )
        },
    );
    // The unvisited suffix's owner closures and computed values have already
    // dropped. Their real interests are restored by the original claim custody.
    for member in &mut frontier.members {
        if result.is_err() && matches!(member.state, MemberState::Prepared) {
            member.state = MemberState::Inspect;
        }
    }
    let stopped = result.err();
    Ok(WaveProgress {
        moved,
        prepared: true,
        stopped,
        queued_stops,
    })
}
