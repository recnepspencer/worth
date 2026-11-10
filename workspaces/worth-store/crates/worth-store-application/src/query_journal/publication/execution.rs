//! Drive the retained original under the current receiver request.
use super::*;
impl QueryJournalPublication {
    pub(in crate::query_journal) fn advance(
        &mut self,
        serving: &ServingPhysicalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> QueryJournalNativeProgress {
        use QueryJournalNativeProgress as Outcome;
        self.resume();
        if let Some(outcome) = self.outcome() {
            return outcome;
        }
        let journal = match journal(serving, self.journal) {
            Ok(journal) => journal,
            Err(cause) => return Outcome::Refused(cause),
        };
        let run = |_: Option<&ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|_| {
                    let (payload, view) = {
                        let mut state = self.state();
                        (state.payload.take(), state.view.take())
                    };
                    if let Some(payload) = payload {
                        match journal.start_publication(
                            payload,
                            view.expect("original payload controls retained"),
                        ) {
                            OrderedJournalPublicationStart::Ready {
                                operation,
                                recovery,
                            } => self.install(operation, recovery),
                            OrderedJournalPublicationStart::AlreadyCommitted(committed) => {
                                self.accept(committed)
                            }
                            OrderedJournalPublicationStart::Refused(cause) => {
                                self.state().endpoint = Endpoint::Refused(cause)
                            }
                        }
                    }
                    if self.state().invalid {
                        self.stop();
                    } else {
                        self.step(&journal);
                    }
                })
                .map(|_| ())
        };
        let result = if request.is_leased() {
            request.in_scope(run).and_then(|result| result)
        } else {
            run(None)
        };
        self.resume();
        if let Some(outcome) = self.outcome() {
            return outcome;
        }
        if let Err(cause) = result {
            self.stop();
            self.resume();
            if let Some(outcome) = self.outcome() {
                return outcome;
            }
            return Outcome::Refused(QueryJournalDenial::Execution(cause));
        }
        Outcome::Pending
    }
}
