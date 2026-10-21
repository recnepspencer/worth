//! One wave: owner preparation, owned compute dispatch, consuming ordered apply.
use super::*;
use super::member_refusal::RefusalCustody;
use super::super::super::super::required_continuations::WaveCheckpointTurn;
use super::readiness_collection;
use super::readiness_collection::MemberStop;
use super::super::selection::committed_ready;
use super::super::super::super::RequiredFreshOutcome;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::canonical_apply::CanonicalApplyCursor;

pub(super) struct WaveProgress {
    pub moved: bool,
    pub delivery: Option<super::super::super::super::required_continuations::DeliveredAwaited>,
    pub deferred: bool,
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
    )?;
    let prepared = match prepared {
        readiness_collection::CollectedRequiredWave::Frozen(frozen) => {
            use super::frozen_collection::FrozenRequiredWave;
            let mut delivered = None;
            match frozen {
                FrozenRequiredWave::Inspected(inspected) => {
                    if let Some(inspected) = inspected {
                        use super::frozen_collection::FrozenTruth;
                        frontier.members[0].state = match inspected {
                            FrozenTruth::Current(proof) => MemberState::Current(Some(proof)),
                            FrozenTruth::NotCurrent => MemberState::Pending,
                        };
                    }
                }
                FrozenRequiredWave::Delivery(
                    super::frozen_collection::SuccessorCustody::Caller(mut delivery),
                ) => {
                    let contacts = (wave.target == RequiredWaveTarget::Caller)
                        .then_some(&mut demand.producer_contacts_in_this_demand);
                    match delivery.deliver(
                        phase,
                        runtime,
                        principal,
                        request,
                        wave.branch,
                        contacts,
                        performed,
                    ) {
                        Ok(receipt) => delivered = receipt,
                        Err(stop) => {
                            RefusingMember {
                                registry: &runtime.output_demands,
                                member: &mut frontier.members[0],
                                row: RefusalRow::HeldHead(delivery.key()),
                                custody: RefusalCustody::<Schema>::Caller,
                            }
                            .end(
                                stop,
                                member_stop::StopRecording::member(),
                                queue,
                            )?;
                        }
                    }
                    if delivered.is_none() {
                        frontier.members[0].state = MemberState::Pending;
                    }
                }
                FrozenRequiredWave::Delivery(
                    super::frozen_collection::SuccessorCustody::Registry(delivery),
                ) => {
                    drop(caller_storage);
                    drop(queue_storage);
                    delivered = delivery.deliver(
                        phase,
                        runtime,
                        principal,
                        request,
                        wave.branch,
                        &mut demand.required_continuations,
                        performed,
                    )?;
                    if delivered.is_none() {
                        frontier.members[0].state = MemberState::Pending;
                    }
                }
            }
            return Ok(WaveProgress {
                moved: false,
                delivery: delivered,
                deferred: false,
                prepared: false,
                stopped: None,
                queued_stops: Vec::new(),
            });
        }
        readiness_collection::CollectedRequiredWave::Live(prepared) => prepared,
    };
    if prepared.members.is_empty() {
        return Ok(WaveProgress {
            moved: false,
            delivery: None,
            deferred: false,
            prepared: false,
            stopped: None,
            queued_stops: Vec::new(),
        });
    }
    let (keys, plans): (Vec<_>, Vec<_>) = prepared.members.into_iter().unzip();

    let keys = worth_proof::CanonicalUniqueVec::try_from_sorted_unique(keys)
        .map_err(|_| foreign_denial())?;
    let execution = phase
        .execution_for(&runtime.product_runtime)
        .map_err(|_| foreign_denial())?;
    let admitted = runtime
        .output_demands
        .admit_prepared_wave(&wave.positioned, keys, plans)?;
    let cursor = CanonicalApplyCursor::compute_admitted(admitted, execution);
    let mut moved = false;
    let mut deferred = false;
    let mut queued_stops = Vec::new();
    let result = cursor.apply(
        |key, outcome| {
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
                // Unlike held::resume, this loop owns installation of a fresh slot;
                // both loops obtain their checkpoint turns from the paid owner.
                loop {
                    let successor = storage.last_mut();
                    let stage = match successor.checkpoint().inspect(&runtime.output_demands) {
                        Ok(WaveCheckpointTurn::Current(ready)) => {
                            runtime
                                .output_demands
                                .clear_required_stop(successor.interest().key());
                            moved |= committed_ready(ready.ready())
                                .map_err(|stop| MemberStop::member(index, stop))?;
                            member.ready.rebind_checkpoint(ready);
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
                        Ok(WaveCheckpointTurn::Awaiting(stage)) => stage,
                    };
                    match stage.advance(phase, runtime, request) {
                        Ok(true) => continue,
                        Ok(false) => {
                            member.state = MemberState::Pending;
                            // Already prepared slots still apply in canonical order.
                            // No later publication or credit may refresh this advance.
                            deferred = true;
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
            // Native cleanup is unique custody, not a queued diagnostic. Stop
            // ordered apply before another member can create a second owner.
            let applied = match applied {
                Err(mut failure) => {
                    if let Some(cleanup) = failure.stop.take_managed_cleanup() {
                        failure.stop = failure.stop.with_managed_cleanup(cleanup);
                        return Err(failure);
                    }
                    Err(failure)
                }
                result => result,
            };
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
                        .reinspection(frontier.members[failure.index].key(), key.inspection)
                        .map_err(|stop| MemberStop::member(failure.index, stop))?;
                    if credit.is_some()
                        && let Some(slot) = frontier.members[failure.index].queued
                    {
                        queue.discard_selection(slot);
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
    let stopped = match result {
        Err(mut failure) => {
            if let Some(cleanup) = failure.stop.take_managed_cleanup() {
                failure.stop = failure.stop.with_managed_cleanup(cleanup);
                let (index, stop, recording) = failure.into_parts();
                recording.record_if_needed(
                    &runtime.output_demands,
                    frontier.members[index].key(),
                    &stop,
                );
                // Return before caller acceptance, reselection or requested
                // dependency bookkeeping can fail and replace this owner.
                return Err(stop);
            }
            Some(failure)
        }
        Ok(()) => None,
    };
    Ok(WaveProgress {
        moved,
        delivery: None,
        deferred,
        prepared: true,
        stopped,
        queued_stops,
    })
}
