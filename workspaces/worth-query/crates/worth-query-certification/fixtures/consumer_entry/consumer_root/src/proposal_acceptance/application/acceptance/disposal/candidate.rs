//! A retired candidate transfers its actual cleanup; it never enters close twice.
use super::*;
pub(super) use crate::proposal_acceptance::application::candidate::CloseFailure;
pub(super) fn closed<'host>(
    result: Result<Closed, CloseFailure<'host>>,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    match result {
        Ok(closed) => {
            let handoff = closed.handoff;
            (
                None,
                Ok(DisposalEvent::Disposed(Disposed {
                    _receipt: Receipt::Candidate { _closed: closed },
                    handoff,
                })),
            )
        }
        Err(failure) => (
            Some(Phase::Disposing(State::Candidate(failure))),
            Ok(DisposalEvent::Retained(DisposalClass::Candidate)),
        ),
    }
}
pub(super) fn retry<'host>(
    failure: CloseFailure<'host>,
    host: &'host Host,
    caller: &Caller<'_>,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    match failure.into_retired_cleanup() {
        Ok((owner, handoff)) => cleanup(owner, handoff, host, caller),
        Err(failure) => closed(failure.staging.close(caller)),
    }
}
pub(super) fn cleanup<'host>(
    owner: WorthQueryApplicationProductBranchCleanup,
    retirement_handoff: Option<WorthQueryAdvancementDenial>,
    host: &'host Host,
    caller: &Caller<'_>,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    let mut owner = Some(owner);
    let mut result = None;
    let advancement = host.with_application_advancement(caller.scope, |_| {
        result = Some(owner.take().expect("original cleanup").retry());
    });
    match (result, advancement) {
        (Some(Ok(receipt)), handoff) => (
            None,
            Ok(DisposalEvent::Disposed(Disposed {
                _receipt: Receipt::Cleanup {
                    _receipt: receipt,
                    _retirement_handoff: retirement_handoff,
                },
                handoff: handoff.err(),
            })),
        ),
        (Some(Err(failure)), handoff) => {
            let state = State::CleanupFailed {
                failure,
                retirement_handoff,
            };
            (
                Some(Phase::Disposing(state)),
                handoff.map(|_| DisposalEvent::Retained(DisposalClass::RetiredCleanup)),
            )
        }
        (None, Err(cause)) => (
            Some(Phase::Disposing(State::Cleanup {
                owner: owner.expect("callback did not consume cleanup"),
                retirement_handoff,
            })),
            Err(cause),
        ),
        (None, Ok(())) => unreachable!("successful advancement invoked cleanup"),
    }
}
