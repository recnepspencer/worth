//! Loan the original physical endpoint; retain its recovery before any caller work.
use super::*;
impl HomeIntentPublication {
    pub(super) fn resume(&self) {
        let recovery = {
            let mut state = self.state();
            if matches!(
                state.endpoint,
                Some(
                    Endpoint::Ready(_)
                        | Endpoint::Stopped(_)
                        | Endpoint::Committed(_)
                        | Endpoint::Refused(_)
                )
            ) {
                return;
            }
            state.recovery.take()
        };
        let Some(recovery) = recovery else {
            return;
        };
        match recovery.resume() {
            OrderedJournalPublicationResume::Committed(committed) => self.committed(committed),
            OrderedJournalPublicationResume::Ready {
                operation,
                recovery,
            } => self.install(operation, recovery),
            OrderedJournalPublicationResume::Stopped {
                operation,
                recovery,
            } => {
                let mut state = self.state();
                state.recovery = Some(recovery);
                state.endpoint = Some(Endpoint::Stopped(operation));
            }
            OrderedJournalPublicationResume::Busy(recovery) => {
                self.state().recovery = Some(recovery)
            }
            OrderedJournalPublicationResume::Terminal(observed) => {
                let valid = self.matches(observed.source());
                let mut state = self.state();
                state.invalid |= !valid;
                state.endpoint = Some(Endpoint::Terminal(observed));
            }
        }
    }
    pub(super) fn step(&self, journal: &OrderedJournal<'_>) {
        let endpoint = self.state().endpoint.replace(Endpoint::InCaller);
        let replacement = match endpoint {
            Some(Endpoint::Ready(operation)) => match operation.step(journal) {
                OrderedJournalPublicationStep::Pending(operation) => {
                    Some(Endpoint::Ready(operation))
                }
                OrderedJournalPublicationStep::Completed(committed) => {
                    self.committed(committed);
                    return;
                }
                OrderedJournalPublicationStep::Failed(custody) => {
                    Some(Endpoint::Stopped(custody.into_stopped()))
                }
            },
            Some(Endpoint::Stopped(operation)) => match operation.settle(journal) {
                OrderedJournalStoppedPoll::Pending(operation) => Some(Endpoint::Stopped(operation)),
                OrderedJournalStoppedPoll::Settled(observed) => {
                    let valid = self.matches(observed.source());
                    self.state().invalid |= !valid;
                    Some(Endpoint::Terminal(observed))
                }
                OrderedJournalStoppedPoll::Failed(custody) => {
                    Some(Endpoint::Stopped(custody.into_stopped()))
                }
            },
            terminal => terminal,
        };
        self.state().endpoint = replacement;
    }
    pub(super) fn stop(&self) {
        let (recovery, endpoint) = {
            let mut state = self.state();
            (state.recovery.take(), state.endpoint.take())
        };
        let recovery = recovery.map(OrderedJournalPublicationRecovery::stop);
        let endpoint = match endpoint {
            Some(Endpoint::Ready(operation)) => Some(Endpoint::Stopped(operation.stop())),
            other => other,
        };
        let mut state = self.state();
        state.recovery = recovery;
        state.endpoint = endpoint;
    }
}
