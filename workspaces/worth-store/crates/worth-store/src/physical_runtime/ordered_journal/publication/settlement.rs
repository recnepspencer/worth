//! Save actual physical facts before releasing input custody or publishing completion.
use super::{operation::*, outcome::*, *};
use crate::physical_runtime::{
    AnchoredRecordStepSettlementFailure, PhysicalMutationOutcome, PhysicalMutationTerminalFact,
    PhysicalRecordId,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
impl JournalPublicationShared {
    pub(in crate::physical_runtime) fn mark_terminal_visible(&self) {
        self.state().visibility = JournalTerminalVisibility::Published;
    }
    pub(super) fn visible_terminal(&self) -> Option<Arc<OrderedJournalPublicationTerminal>> {
        let s = self.state();
        if s.visibility == JournalTerminalVisibility::AwaitingBasePublication {
            None
        } else {
            s.terminal.clone()
        }
    }
    pub(super) fn readmit_committed(&self) -> Option<OrderedJournalCommitted> {
        let terminal = self.visible_terminal()?;
        let expected = self.state().mutation;
        if self.state().visibility != JournalTerminalVisibility::Published
            || terminal.failure.is_some()
            || terminal.source != self.source
            || terminal.mutation != expected
        {
            return None;
        }
        let published = terminal.published?;
        let entry = published.entry();
        let Some(PhysicalMutationOutcome::Completed(actual)) = terminal.outcome.as_ref() else {
            return None;
        };
        let [record] = actual.persisted_records() else {
            return None;
        };
        if Some(actual.mutation_identity()) != expected
            || actual.mutation_identity().runtime_identity() != self.source.runtime
            || entry.position() != self.source.position
            || entry.payload_sha256() != self.source.digest
            || entry.payload_bytes() != self.source.bytes
            || entry.record() != PhysicalRecordId::from_persisted(*record)
        {
            return None;
        }
        Some(OrderedJournalCommitted { terminal })
    }
    pub(in crate::physical_runtime) fn finish(
        &self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<AnchoredRecordStepSettlementFailure>,
    ) {
        let (mutation, duplicate, retired) = {
            let mut s = self.state();
            if let Some(terminal) = s.terminal.as_ref() {
                if s.visibility == JournalTerminalVisibility::AwaitingBasePublication
                    && let Some(cause) = failure
                {
                    let updated = OrderedJournalPublicationTerminal {
                        source: terminal.source,
                        mutation: terminal.mutation,
                        outcome: s.fact.as_ref().map(|original| original.outcome()),
                        published: terminal.published,
                        failure: terminal
                            .failure
                            .or(Some(OrderedJournalTerminalFailure::Settlement(cause))),
                    };
                    s.terminal = Some(Arc::new(updated));
                }
                return;
            }
            if s.settling {
                return;
            }

            s.fact = Some(fact.clone());
            s.settling = true;
            (
                s.mutation,
                s.unresolved_duplicate,
                (s.input.take(), s.interruption.take()),
            )
        };
        let outcome = fact.outcome();
        let published = match &outcome {
            PhysicalMutationOutcome::Completed(done) => match done.persisted_records() {
                [record] => Some(OrderedJournalPublished::Committed(OrderedJournalEntry {
                    position: self.source.position,
                    record: PhysicalRecordId::from_persisted(*record),
                    payload_sha256: self.source.digest,
                    payload_bytes: self.source.bytes,
                })),
                _ => None,
            },
            _ => None,
        };
        let mut failure = if duplicate {
            Some(OrderedJournalTerminalFailure::UnresolvedDuplicate)
        } else {
            failure.map(OrderedJournalTerminalFailure::Settlement)
        };
        if matches!(outcome, PhysicalMutationOutcome::Completed(_)) && published.is_none() {
            failure = failure.or(Some(OrderedJournalTerminalFailure::CompletedWithoutRecord));
        }
        if catch_unwind(AssertUnwindSafe(|| drop(retired))).is_err() {
            failure = failure.or(Some(OrderedJournalTerminalFailure::Settlement(
                AnchoredRecordStepSettlementFailure::ProtectionReleasePanicked,
            )));
        }
        let unresolved =
            matches!(outcome, PhysicalMutationOutcome::Indeterminate(_)) || failure.is_some();
        let terminal = Arc::new(OrderedJournalPublicationTerminal {
            source: self.source,
            mutation,
            outcome: Some(outcome),
            published,
            failure,
        });
        if unresolved && let Some(owner) = self.owner.upgrade() {
            owner
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .unresolved
                .insert(self.source.journal, self.source.position);
        }
        let mut s = self.state();
        s.terminal = Some(terminal);
        s.live = false;
        s.settling = false;
    }
    pub(super) fn observe_existing(&self) {
        let (existing, cause) = {
            let s = self.state();
            (s.existing.clone(), s.refusal)
        };
        if let Some(existing) = existing
            && let Some(fact) = existing.terminal()
        {
            self.finish(
                &fact,
                cause.map(AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused),
            );
            let complete = self.state().terminal.is_some();
            if complete {
                self.mark_terminal_visible();
                let retired = self.state().existing.take();
                drop(retired);
            }
        }
    }
    pub(super) fn poll_terminal(&self) -> Result<bool, OrderedJournalOperationFailure> {
        self.observe_existing();
        let terminal = self.visible_terminal();
        let (preparing, settling, registered, duplicate, refusal, has_attempt, existing, fact) = {
            let s = self.state();
            (
                s.preparing,
                s.settling,
                s.mutation.is_some(),
                s.unresolved_duplicate,
                s.refusal,
                s.attempt.upgrade().is_some(),
                s.existing.is_some(),
                s.fact.clone(),
            )
        };
        if let Some(terminal) = terminal {
            if let Some(failure) = terminal.failure {
                return Err(match failure {
                    OrderedJournalTerminalFailure::UnresolvedDuplicate => {
                        OrderedJournalOperationFailure::UnresolvedDuplicate
                    }
                    OrderedJournalTerminalFailure::Settlement(c) => {
                        OrderedJournalOperationFailure::Settlement(c)
                    }
                    OrderedJournalTerminalFailure::CompletedWithoutRecord => {
                        Failure::CompletedWithoutRecord.into()
                    }
                });
            }
            return match terminal.outcome.as_ref() {
                Some(PhysicalMutationOutcome::ProvenNoEffect(f)) => {
                    Err(Failure::ProvenNoEffect(f.clone()).into())
                }
                Some(PhysicalMutationOutcome::Indeterminate(f)) => {
                    Err(Failure::Indeterminate(f.clone()).into())
                }
                _ => Ok(true),
            };
        }
        if preparing || settling {
            return Ok(false);
        }
        if duplicate && !existing {
            return Err(OrderedJournalOperationFailure::UnresolvedDuplicate);
        }
        if let Some(cause) = refusal
            && !existing
        {
            return Err(OrderedJournalOperationFailure::Settlement(
                AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(cause),
            ));
        }
        if registered && !has_attempt && !existing {
            if let Some(fact) = fact {
                self.finish(
                    &fact,
                    Some(AnchoredRecordStepSettlementFailure::WorkerUnavailable),
                );
                self.state().visibility = JournalTerminalVisibility::WorkerLost;
                return self.poll_terminal();
            }
            self.retire_unavailable();
            return Err(OrderedJournalOperationFailure::Settlement(
                AnchoredRecordStepSettlementFailure::WorkerUnavailable,
            ));
        }
        Ok(!registered)
    }
    fn retire_unavailable(&self) {
        let retired = {
            let mut s = self.state();
            s.live = false;
            s.stopped = true;
            (s.input.take(), s.interruption.take())
        };
        drop(retired);
        if let Some(owner) = self.owner.upgrade() {
            owner
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .unresolved
                .insert(self.source.journal, self.source.position);
        }
    }
    pub(super) fn wait(&self) {
        let attempt = self.state().attempt.upgrade();
        if let Some(a) = attempt {
            crate::physical_runtime::PhysicalMutationHandle::new(&a).wait();
        }
    }
}
