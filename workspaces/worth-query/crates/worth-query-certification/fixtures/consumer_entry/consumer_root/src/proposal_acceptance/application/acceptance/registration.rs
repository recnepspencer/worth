//! Native metadata completion stays owned until a fresh claim read and successful close.
use super::*;
pub(super) fn advance<'host>(
    phase: Phase<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
) -> (Option<Phase<'host>>, Result<Event, Failure>) {
    let request = storage.request(caller);
    let completed = match phase {
        Phase::Claim(operation) => match operation.advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(owner) => {
                return (Some(Phase::Claim(owner)), Ok(Event::Progress))
            }
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                return (
                    Some(Phase::Claim(operation)),
                    Err(Failure::MetadataAdvance(denial)),
                )
            }
            WorthQuerySourceOnlyAdvance::Terminal(owner) => {
                return (Some(Phase::ClaimTerminal(owner)), Ok(Event::Stopped))
            }
            WorthQuerySourceOnlyAdvance::Completed(owner) => owner,
        },
        Phase::ClaimCompleted(owner) => owner,
        _ => unreachable!("registration phases"),
    };
    let claim = match storage.read_claim(caller, &loaded.input(key)) {
        Ok(claim) => claim,
        Err(cause) => {
            return (
                Some(Phase::ClaimCompleted(completed)),
                Err(Failure::ClaimRead(cause)),
            )
        }
    };
    match completed.close_with_evidence(&request) {
        Ok((_closed, evidence)) => (
            Some(Phase::Registered(Registration { claim, evidence })),
            Ok(Event::Progress),
        ),
        Err((owner, cause)) => (
            Some(Phase::ClaimCompleted(owner)),
            Err(Failure::OperationClose(cause)),
        ),
    }
}
