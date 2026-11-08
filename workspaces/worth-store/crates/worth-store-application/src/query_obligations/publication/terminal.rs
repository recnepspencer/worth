//! Classify actual physical evidence without converting unknown fate to no-effect.
use super::*;
impl QueryObligationPublication {
    pub(super) fn proven_no_effect(&self, observed: &OrderedJournalPublicationObservation) -> bool {
        let Some(terminal) = observed.terminal() else {
            return false;
        };
        if !self.matches(terminal.source()) || terminal.failure().is_some() {
            return false;
        }
        match terminal.physical_outcome() {
            Some(PhysicalMutationOutcome::ProvenNoEffect(_)) => true,
            Some(
                PhysicalMutationOutcome::Completed(_) | PhysicalMutationOutcome::Indeterminate(_),
            )
            | None => false,
        }
    }
    pub(super) fn outcome(&self) -> Option<QueryObligationNativeProgress> {
        use QueryObligationNativeProgress as Outcome;
        let state = self.state();
        if state.invalid {
            return Some(Outcome::Unresolved);
        }
        match &state.endpoint {
            Endpoint::Committed(committed) => {
                let source = committed.source();
                Some(Outcome::Committed {
                    position: source.position(),
                    digest: source.payload_sha256(),
                })
            }
            Endpoint::Refused(cause) => {
                if start_proves_no_effect(cause) {
                    Some(Outcome::ProvenNoEffect {
                        position: self.position,
                        digest: self.digest,
                    })
                } else {
                    Some(Outcome::Unresolved)
                }
            }
            Endpoint::NoEffect(observed) => {
                let source = observed.source();
                Some(Outcome::ProvenNoEffect {
                    position: source.position(),
                    digest: source.payload_sha256(),
                })
            }
            Endpoint::NoEffectBeforeStart => Some(Outcome::ProvenNoEffect {
                position: self.position,
                digest: self.digest,
            }),
            Endpoint::Terminal(observed) => {
                let _ = observed.terminal();
                Some(Outcome::Unresolved)
            }
            Endpoint::Unstarted
            | Endpoint::InCaller
            | Endpoint::Ready(_)
            | Endpoint::Stopped(_) => None,
        }
    }
    pub(super) fn step(&self, journal: &OrderedJournal<'_>) {
        let endpoint = {
            let mut state = self.state();
            std::mem::replace(&mut state.endpoint, Endpoint::InCaller)
        };
        let replacement = match endpoint {
            Endpoint::Ready(operation) => match operation.step(journal) {
                OrderedJournalPublicationStep::Pending(operation) => Endpoint::Ready(operation),
                OrderedJournalPublicationStep::Completed(committed) => {
                    self.accept(committed);
                    return;
                }
                OrderedJournalPublicationStep::Failed(failure) => {
                    Endpoint::Stopped(failure.into_stopped())
                }
            },
            Endpoint::Stopped(operation) => match operation.settle(journal) {
                OrderedJournalStoppedPoll::Pending(operation) => Endpoint::Stopped(operation),
                OrderedJournalStoppedPoll::Settled(observed) => {
                    self.state().invalid |= !self.matches(observed.source());
                    // Settled custody with no registered mutation is the real
                    // pre-start no-effect terminal. A completed mutation still
                    // needs its retained recovery to issue committed custody.
                    if observed.mutation_identity().is_none() && observed.terminal().is_none() {
                        Endpoint::NoEffect(observed)
                    } else {
                        Endpoint::Terminal(observed)
                    }
                }
                OrderedJournalStoppedPoll::Failed(failure) => {
                    let observed = failure.observe();
                    if self.proven_no_effect(&observed) {
                        drop(failure.into_stopped());
                        Endpoint::NoEffect(observed)
                    } else {
                        Endpoint::Stopped(failure.into_stopped())
                    }
                }
            },
            same @ (Endpoint::Unstarted
            | Endpoint::InCaller
            | Endpoint::Committed(_)
            | Endpoint::Refused(_)
            | Endpoint::Terminal(_)
            | Endpoint::NoEffect(_)
            | Endpoint::NoEffectBeforeStart) => same,
        };
        self.state().endpoint = replacement;
    }
}

/// Refusing a new endpoint does not establish absence of an earlier original.
fn start_proves_no_effect(cause: &OrderedJournalOperationFailure) -> bool {
    use OrderedJournalOperationFailure as O;
    use OrderedJournalPublicationFailure as F;
    match cause {
        O::OriginalRequestStopped(_) | O::Busy | O::Stopped => true,
        // The physical failure enum is non-exhaustive. Only its known,
        // explicitly pre-effect refusals establish this closed fact.
        O::Publication(failure) => matches!(
            failure,
            F::EmptyPayload
                | F::PositionGap { .. }
                | F::PositionConflict(_)
                | F::AnchorTableFull
                | F::Batch(_)
                | F::Idempotency(_)
                | F::Preparation(_)
                | F::ProvenNoEffect(_)
                | F::Read(_)
        ),
        O::CompletionReadmissionMismatch
        | O::UnresolvedDuplicate
        | O::ForeignReceiver
        | O::OwnerReleased
        | O::Panicked
        | O::Settlement(_) => false,
    }
}
