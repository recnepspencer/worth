//! Delivery retains the committed result while the original previous-head owner settles.
use super::*;
pub(super) fn advance<'host>(
    phase: Phase<'host>,
    caller: &Caller<'_>,
) -> (Option<Phase<'host>>, Result<Event, Failure>) {
    match phase {
        Phase::PreviousClose { failure, result } => match failure.into_retired() {
            Ok(owner) => (
                Some(Phase::RetiredPrevious { owner, result }),
                Ok(Event::Progress),
            ),
            Err(failure) => match failure.adopted.close_previous(caller) {
                Ok(performed) => (
                    None,
                    Ok(Event::Completed(Accepted {
                        result,
                        performed: FinishedAdoption::PreviousClosed(performed),
                    })),
                ),
                Err(failure) => (
                    Some(Phase::PreviousClose { failure, result }),
                    Ok(Event::Stopped),
                ),
            },
        },
        Phase::RetiredPrevious { owner, result } => match owner.retry(caller) {
            Ok(performed) => (
                None,
                Ok(Event::Completed(Accepted {
                    result,
                    performed: FinishedAdoption::RetiredCleanupSettled(performed),
                })),
            ),
            Err(owner) => (
                Some(Phase::RetiredPrevious { owner, result }),
                Ok(Event::Stopped),
            ),
        },
        _ => unreachable!("previous-head recovery phases"),
    }
}
