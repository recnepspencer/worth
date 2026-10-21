//! Route retained successor custody before inspecting its stopped predecessor.
use super::super::super::super::required_continuations::PreparedContinuationWave;
use super::super::super::super::required_continuations::RequiredContinuations;
use super::super::frontier::RequiredMember;
use super::*;

pub(super) fn defer<Schema: ApplicationSchema + 'static>(
    member: &mut RequiredMember,
    caller: &PreparedContinuationWave<'_, Schema>,
    queue: &PreparedContinuationWave<'_, Schema>,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    let Some(selected) = member.ready.bound_ready() else {
        return Ok(false);
    };
    let storage = if member.caller { caller } else { queue };
    if !storage.retains_refresh(selected)? {
        return Ok(false);
    }
    member.state = MemberState::RetainedRefresh;
    Ok(true)
}

/// Keep the genuine predecessor pin while its replacement is owned, including
/// across a queued member's basis change. It is never a currentness proof.
pub(super) fn preserves_selection<Schema: ApplicationSchema + 'static>(
    member: &RequiredMember,
    caller: &RequiredContinuations<Schema>,
    queue: &RequiredContinuations<Schema>,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    let Some(selected) = member.ready.bound_ready() else {
        return Ok(false);
    };
    let storage = if member.caller { caller } else { queue };
    storage
        .refresh_position(selected)
        .map(|position| position.is_some())
}

/// Only the caller target attributes its own handler attempts to the public
/// caller count. A requested child keeps its custody without charging its parent.
pub(super) fn custody<'a, Schema: ApplicationSchema>(
    caller: bool,
    target: RequiredWaveTarget,
    caller_entries: &'a mut RequiredContinuations<Schema>,
    contacts: &'a mut usize,
    queue: &'a mut RequiredContinuations<Schema>,
) -> ContinuationCustody<'a, Schema> {
    if caller && target == RequiredWaveTarget::Caller {
        ContinuationCustody::Caller(caller_entries, contacts)
    } else if caller {
        ContinuationCustody::Queue(caller_entries)
    } else {
        ContinuationCustody::Queue(queue)
    }
}
