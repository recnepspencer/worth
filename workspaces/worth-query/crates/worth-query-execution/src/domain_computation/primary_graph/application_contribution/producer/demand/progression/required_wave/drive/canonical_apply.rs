//! Consume a Current root; hold remaining unfinished queue claims.
use super::*;

pub(super) fn finish<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    wave: &RequiredWaveSelection<'_, Schema>,
    frontier: &mut RequiredFrontier,
    queue: &mut QueuedRequiredRoots,
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
        return Ok(Some(return_requested(settlement)));
    }
    if !queue.draining() {
        match accept_current_caller(runtime, demand, wave, frontier)? {
            CallerAcceptance::Accepted(proof) => queue.finish_caller(proof),
            CallerAcceptance::Deferred => queue.defer_caller(),
        }
    }
    for member in &frontier.members {
        if let Some(index) = member.queued {
            queue.hold_root(index, None)?;
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
) -> Result<CallerAcceptance, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let caller = &mut frontier.members[0];
    let MemberState::Current(decision) = &mut caller.state else {
        return Ok(CallerAcceptance::Deferred);
    };
    let proof = decision.take().expect("the caller owns its Current proof");
    match super::super::caller::finish_current_caller(
        runtime,
        demand,
        super::super::caller::CurrentCaller {
            anchor_ready: &wave.anchor_ready,
            selected: caller.ready.selected(),
            continues_caller: caller.caller_successor,
            caller_current: !caller.caller_successor,
            current_contacts: demand.producer_contacts_in_this_demand,
        },
        proof,
    )? {
        Some(proof) => {
            frontier.members[0].state = MemberState::AcceptedCaller;
            Ok(CallerAcceptance::Accepted(proof))
        }
        _ => {
            caller.state = MemberState::Pending;
            Ok(CallerAcceptance::Deferred)
        }
    }
}

/// Native Current proof consumption is immediate; only promotion may defer.
pub(in super::super) enum CallerAcceptance {
    Accepted(performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>),
    Deferred,
}

fn return_requested(
    proof: performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>,
) -> WorthQueryOutputDemandAdvance {
    WorthQueryOutputDemandAdvance::Settled(proof.into_value())
}
