//! Save actual physical facts before releasing input custody or publishing completion.
use super::{operation::*, outcome::*, *};
use crate::physical_runtime::{
    AnchoredRecordStepSettlementFailure, PhysicalMutationOutcome, PhysicalMutationTerminalFact,
    PhysicalRecordId,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
impl JournalPublicationShared {
    pub(in crate::physical_runtime) fn finish(
        &self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<AnchoredRecordStepSettlementFailure>,
    ) {
        let (mutation, retired) = {
            let mut s = self.state();
            if s.terminal.is_some() || s.settling {
                return;
            }
            s.fact = Some(fact.clone());
            s.settling = true;
            (s.mutation, (s.input.take(), s.interruption.take()))
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
        let mut failure = failure.map(OrderedJournalTerminalFailure::Settlement);
        if matches!(outcome, PhysicalMutationOutcome::Completed(_)) && published.is_none() {
            failure = Some(OrderedJournalTerminalFailure::CompletedWithoutRecord);
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
            let retired = self.state().existing.take();
            drop(retired);
        }
    }
    pub(super) fn poll_terminal(&self) -> Result<bool, OrderedJournalOperationFailure> {
        self.observe_existing();
        let (terminal, preparing, submitted, refusal, has_attempt, existing, fact) = {
            let s = self.state();
            (
                s.terminal.clone(),
                s.preparing,
                s.submitted,
                s.refusal,
                s.attempt.upgrade().is_some(),
                s.existing.is_some(),
                s.fact.clone(),
            )
        };
        if let Some(terminal) = terminal {
            if let Some(failure) = terminal.failure {
                return Err(match failure {
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
        if preparing {
            return Ok(false);
        }
        if let Some(cause) = refusal
            && !existing
        {
            return Err(OrderedJournalOperationFailure::Settlement(
                AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(cause),
            ));
        }
        if submitted && !has_attempt && !existing {
            if let Some(fact) = fact {
                self.finish(
                    &fact,
                    Some(AnchoredRecordStepSettlementFailure::WorkerUnavailable),
                );
                return self.poll_terminal();
            }
            self.retire_unavailable();
            return Err(OrderedJournalOperationFailure::Settlement(
                AnchoredRecordStepSettlementFailure::WorkerUnavailable,
            ));
        }
        Ok(!submitted)
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
