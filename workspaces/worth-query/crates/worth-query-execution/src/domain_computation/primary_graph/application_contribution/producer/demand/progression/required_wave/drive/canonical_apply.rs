//! Consume a Current root; hold remaining unfinished queue claims.
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
            Some(proof) => Some(return_requested(proof)),
            None => {
                frontier.members[0].state = MemberState::Inspect;
                None
            }
        });
    }
    if !queue.draining() {
        match accept_current_caller(runtime, demand, wave, frontier, performed, admission)? {
            CallerAcceptance::Accepted(outcome) => queue.finish_caller(outcome),
            CallerAcceptance::Deferred => {
                queue.finish_caller(WorthQueryOutputDemandAdvance::Pending)
            }
            CallerAcceptance::Reinspect => return Ok(None),
        }
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
) -> Result<CallerAcceptance, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let caller = &mut frontier.members[0];
    let MemberState::Current(decision) = &mut caller.state else {
        return Ok(CallerAcceptance::Deferred);
    };
    let decision = decision
        .take()
        .expect("the caller owns its Current decision");
    let proof = match consume_caller_decision(decision, &performed.borrow()) {
        Ok(proof) => proof,
        Err(refusal) => {
            caller.state = MemberState::Inspect;
            return Ok(refusal);
        }
    };
    if let Some(settlement) = super::super::caller::finish_current_caller(
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
        admission,
    )? {
        let outcome = WorthQueryOutputDemandAdvance::Settled(settlement);
        frontier.members[0].state = MemberState::AcceptedCaller;
        Ok(CallerAcceptance::Accepted(outcome))
    } else {
        caller.state = MemberState::Pending;
        Ok(CallerAcceptance::Deferred)
    }
}

/// Distinguish a refused proof from a genuinely deferred caller promotion.
pub(in super::super) enum CallerAcceptance {
    Accepted(WorthQueryOutputDemandAdvance),
    Deferred,
    Reinspect,
}

pub(in super::super) fn consume_caller_decision<T>(
    decision: performed::CertifiedDecision<T>,
    owner: &performed::PerformedMembers,
) -> Result<performed::CurrentProof<T>, CallerAcceptance> {
    decision
        .into_current(owner)
        .ok_or(CallerAcceptance::Reinspect)
}

fn return_requested(
    proof: performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>,
) -> WorthQueryOutputDemandAdvance {
    WorthQueryOutputDemandAdvance::Settled(proof.into_value())
}
