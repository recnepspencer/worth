//! Close the genuine metadata owner on its original routing branch without executing an input.
use super::*;
pub(in crate::proposal_acceptance::application::acceptance) enum Owner {
    Pending(Box<ClaimOperation>),
    Completed(Box<ClaimCompleted>),
    Terminal(Box<ClaimTerminal>),
}
pub(super) fn close<'host>(
    owner: Owner,
    host: &'host Host,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
) -> (
    Option<Phase<'host>>,
    Result<DisposalEvent, WorthQueryAdvancementDenial>,
) {
    let request = host
        .request(&caller.principal, caller.scope)
        .on_branch(loaded.source());
    let closed = match owner {
        Owner::Pending(owner) => (*owner)
            .close(&request)
            .map_err(|(owner, cause)| (Owner::Pending(Box::new(owner)), cause)),
        Owner::Completed(owner) => (*owner)
            .close(&request)
            .map_err(|(owner, cause)| (Owner::Completed(Box::new(owner)), cause)),
        Owner::Terminal(owner) => (*owner)
            .close(&request)
            .map_err(|(owner, cause)| (Owner::Terminal(Box::new(owner)), cause)),
    };
    match closed {
        Ok(closed) => (
            None,
            Ok(DisposalEvent::Disposed(Disposed {
                _receipt: Receipt::Operation { _closed: closed },
                handoff: None,
            })),
        ),
        Err((owner, cause)) => (
            Some(Phase::Disposing(State::Metadata { owner, cause })),
            Ok(DisposalEvent::Retained(DisposalClass::Operation)),
        ),
    }
}
