//! Accept only a Current caller, and acknowledge only Current queued roots.
use super::*;

pub(super) fn finish<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    wave: &RequiredWaveSelection<'_, Schema>,
    frontier: &mut RequiredFrontier,
    queue: &mut QueuedRequiredRoots,
    performed: &std::cell::RefCell<performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    if wave.target == RequiredWaveTarget::Requested {
        let MemberState::Current(decision) = &mut frontier.members[0].state else {
            return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
        };
        let settlement = decision
            .take()
            .expect("the requested target owns its decision");
        return Ok(match settlement.into_current(&performed.borrow()) {
            Some(settlement) => Some(WorthQueryOutputDemandAdvance::Settled(settlement)),
            None => {
                frontier.members[0].state = MemberState::Inspect;
                None
            }
        });
    }
    if !queue.draining() {
        queue.finish_caller(
            accept_current_caller(runtime, demand, wave, frontier, performed, admission)?
                .unwrap_or(WorthQueryOutputDemandAdvance::Pending),
        );
    }
    for member in &frontier.members {
        if let Some(index) = member.queued {
            queue.hold_root(index, None, admission)?;
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
    performed: &std::cell::RefCell<performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let caller = &mut frontier.members[0];
    let MemberState::Current(decision) = &mut caller.state else {
        return Ok(None);
    };
    let decision = decision
        .take()
        .expect("the caller owns its Current decision");
    let Some(settlement) = decision.into_current(&performed.borrow()) else {
        caller.state = MemberState::Inspect;
        return Ok(None);
    };
    if super::super::caller::finish_current_caller(
        runtime,
        demand,
        super::super::caller::CurrentCaller {
            anchor_ready: &wave.anchor_ready,
            selected: caller.ready.selected(),
            continues_caller: caller.caller_successor,
            caller_current: !caller.caller_successor,
            current_contacts: demand.producer_contacts_in_this_demand,
        },
        admission,
    )? {
        let outcome = WorthQueryOutputDemandAdvance::Settled(settlement);
        frontier.members[0].state = MemberState::AcceptedCaller;
        Ok(Some(outcome))
    } else {
        caller.state = MemberState::Pending;
        Ok(None)
    }
}
