//! Accept only a Current caller, and acknowledge only Current queued roots.
use super::*;

pub(super) fn finish<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    wave: &RequiredWaveSelection<'_, Schema>,
    frontier: &mut RequiredFrontier,
    queue: &mut QueuedRequiredRoots,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    if wave.target == RequiredWaveTarget::Requested {
        return Ok(Some(match &frontier.members[0].state {
            MemberState::Current(settlement) => WorthQueryOutputDemandAdvance::Settled(Arc::clone(
                settlement
                    .as_ref()
                    .expect("the caller owns its Current report"),
            )),
            MemberState::Inspect
            | MemberState::Waiting(_)
            | MemberState::Prepared
            | MemberState::Pending
            | MemberState::AcceptedCaller
            | MemberState::Held(_) => WorthQueryOutputDemandAdvance::Pending,
        }));
    }
    if !queue.draining() {
        queue.finish_caller(
            accept_current_caller(runtime, demand, wave, frontier, admission)?
                .unwrap_or(WorthQueryOutputDemandAdvance::Pending),
        );
    }
    for member in &frontier.members {
        if let Some(index) = member.queued {
            if matches!(member.state, MemberState::Current(_)) {
                queue.acknowledge_root(index, admission)?;
            } else {
                queue.hold_root(index, None, admission)?;
            }
        }
    }
    Ok(queue.take_caller())
}

/// Caller failure returns at once; a queue failure cannot replace an already
/// certified caller outcome. Certification is separate from committing a slot.
pub(super) fn accept_current_caller<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    wave: &RequiredWaveSelection<'_, Schema>,
    frontier: &mut RequiredFrontier,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let caller = &frontier.members[0];
    let MemberState::Current(settlement) = &caller.state else {
        return Ok(None);
    };
    if super::super::caller::finish_current_caller(
        runtime,
        demand,
        super::super::caller::CurrentCaller {
            anchor_ready: &wave.anchor_ready,
            selected: &caller.ready,
            continues_caller: caller.caller_successor,
            caller_current: !caller.caller_successor,
            current_contacts: demand.producer_contacts_in_this_demand,
        },
        admission,
    )? {
        let outcome = WorthQueryOutputDemandAdvance::Settled(Arc::clone(
            settlement
                .as_ref()
                .expect("the caller owns its Current report"),
        ));
        frontier.members[0].state = MemberState::AcceptedCaller;
        Ok(Some(outcome))
    } else {
        Ok(None)
    }
}
