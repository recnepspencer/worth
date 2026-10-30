//! Preparation retains its original creation failure; adoption replaces the actual model head.
use super::*;
pub(super) fn advance<'host>(
    phase: Phase<'host>,
    model: &mut Model<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
) -> (Option<Phase<'host>>, Result<Event, Failure>) {
    match phase {
        Phase::Registered(registration) => {
            match Staging::prepare_frozen(model, storage, caller, loaded, key) {
                Ok(staging) => (
                    Some(Phase::Staging {
                        registration,
                        staging,
                    }),
                    Ok(Event::Progress),
                ),
                Err(PreparationFailure::Created { staging, handoff }) => (
                    Some(Phase::Staging {
                        registration,
                        staging,
                    }),
                    Err(Failure::Advancement(handoff)),
                ),
                Err(failure) => (
                    Some(Phase::PreparingFailed {
                        registration,
                        failure,
                    }),
                    Ok(Event::Stopped),
                ),
            }
        }
        Phase::Staging {
            registration,
            mut staging,
        } => {
            let event = staging.advance(model, caller);
            match event {
                candidate::StageEvent::AllStaged => match staging.finish() {
                    Ok(ready) => (
                        Some(Phase::Ready {
                            registration,
                            ready,
                        }),
                        Ok(Event::Progress),
                    ),
                    Err(staging) => (
                        Some(Phase::Staging {
                            registration,
                            staging,
                        }),
                        Ok(Event::Progress),
                    ),
                },
                event => (
                    Some(Phase::Staging {
                        registration,
                        staging,
                    }),
                    Ok(Event::Staged(event)),
                ),
            }
        }
        Phase::Ready {
            registration,
            ready,
        } => match ready.adopt(model, caller) {
            Ok(adopted) => (
                Some(Phase::Adopted {
                    registration,
                    adopted,
                    delivery: Delivery::Needed,
                }),
                Ok(Event::Adopted),
            ),
            Err(failure) => (
                Some(Phase::Ready {
                    registration,
                    ready: failure.ready,
                }),
                Err(Failure::Adoption {
                    cause: failure.cause,
                    handoff: failure.handoff,
                }),
            ),
        },
        _ => unreachable!("candidate phases"),
    }
}
