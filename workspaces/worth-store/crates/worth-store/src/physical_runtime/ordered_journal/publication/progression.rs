//! One consuming call prepares/submits once or observes that exact accepted step.
use super::{operation::*, outcome::*, *};
use crate::physical_runtime::record_serving::PreparedDurableRecordAppend;
use crate::physical_runtime::{
    AnchoredRecordStepSettlementFailure, PhysicalMutationOutcome, PhysicalMutationTerminalFact,
    PhysicalPreSealCancellationOutcome,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
struct Preparing(Arc<JournalPublicationShared>);
impl Drop for Preparing {
    fn drop(&mut self) {
        let stopped = {
            let mut s = self.0.state();
            s.preparing = false;
            s.stopped
        };
        if stopped {
            self.0.stop();
        }
    }
}
impl OrderedJournalPublicationOperation {
    pub fn step(mut self, journal: &OrderedJournal<'_>) -> OrderedJournalPublicationStep {
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.shared.require_receiver(journal)?;
            let registered = self.shared.state().mutation.is_some();
            if !registered {
                if self.shared.state().stopped {
                    return Err(OrderedJournalOperationFailure::Stopped);
                }
                submit(&self.shared, journal)?;
            }
            self.shared.poll_terminal()
        }));
        match result {
            Ok(Ok(false)) => OrderedJournalPublicationStep::Pending(self),
            Ok(Ok(true)) => {
                let Some(committed) = self.shared.readmit_committed() else {
                    return OrderedJournalPublicationStep::Failed(
                        OrderedJournalPublicationFailureCustody {
                            stopped: self.stop(),
                            cause: OrderedJournalOperationFailure::CompletionReadmissionMismatch,
                        },
                    );
                };
                self.shared.state().endpoint_live = false;
                if let Some(r) = self.custody.take() {
                    let _ = r.terminate(Disposition::Published);
                }
                OrderedJournalPublicationStep::Completed(committed)
            }
            result => {
                let cause = match result {
                    Ok(Err(c)) => c,
                    Err(_) => OrderedJournalOperationFailure::Panicked,
                    Ok(Ok(_)) => unreachable!(),
                };
                OrderedJournalPublicationStep::Failed(OrderedJournalPublicationFailureCustody {
                    stopped: self.stop(),
                    cause,
                })
            }
        }
    }
}
fn submit(
    shared: &Arc<JournalPublicationShared>,
    journal: &OrderedJournal<'_>,
) -> Result<(), OrderedJournalOperationFailure> {
    let (plan, view) = {
        let mut s = shared.state();
        if let Some(stop) = s.interruption.as_ref().and_then(|v| v.check_live().err()) {
            return Err(OrderedJournalOperationFailure::OriginalRequestStopped(stop));
        }
        let plan = s
            .plan
            .take()
            .ok_or(OrderedJournalOperationFailure::Stopped)?;
        s.preparing = true;
        (plan, s.interruption.clone())
    };
    let _preparing = Preparing(shared.clone());
    let prepared = plan
        .into_durable_append()
        .map_err(journal_failure)?
        .prepare_with_interruption(&journal.runtime.record_submission(), view)
        .map_err(journal_failure)?;
    let identity = match &prepared {
        PreparedDurableRecordAppend::Prepared(p) => p.mutation_identity(),
        PreparedDurableRecordAppend::Terminal(fact) => match fact.outcome() {
            PhysicalMutationOutcome::Completed(d) => d.mutation_identity(),
            PhysicalMutationOutcome::ProvenNoEffect(d) => d.mutation_identity(),
            PhysicalMutationOutcome::Indeterminate(d) => d.mutation_identity(),
        },
    };
    let stopped = {
        let mut s = shared.state();
        s.mutation = Some(identity);
        s.stopped
    };
    match prepared {
        PreparedDurableRecordAppend::Prepared(prepared)
            if prepared.disposition()==crate::physical_runtime::PhysicalMutationAdmissionDisposition::DuplicateUnresolved => {
            // Joining identity is not permission to replay its unresolved work.
            let existing=prepared.observe_existing_execution().map(Arc::new);
            let retired={let mut s=shared.state();s.unresolved_duplicate=true;s.existing=existing;(s.input.take(),s.interruption.take())};
            if let Some(owner)=shared.owner.upgrade(){owner.lock().unwrap_or_else(|p|p.into_inner()).unresolved.insert(shared.source.journal,shared.source.position);}
            drop(prepared);drop(retired);
            return Err(OrderedJournalOperationFailure::UnresolvedDuplicate);
        }
        PreparedDurableRecordAppend::Terminal(fact) => {shared.finish(&fact,None);shared.mark_terminal_visible();},
        PreparedDurableRecordAppend::Prepared(prepared) if stopped => {
            match journal
                .runtime
                .record_submission()
                .cancel_prepared_before_group_seal(prepared)
            {
                PhysicalPreSealCancellationOutcome::ProvenNoEffect(fate) => {shared.finish(
                    &PhysicalMutationTerminalFact::from_outcome(
                        PhysicalMutationOutcome::ProvenNoEffect(fate),
                    ),
                    None,
                );shared.mark_terminal_visible();},
                PhysicalPreSealCancellationOutcome::NotCancelled { prepared, cause } => {
                    let existing = prepared.observe_existing_execution().map(Arc::new);
                    let retired = {
                        let mut s = shared.state();
                        s.refusal = Some(cause);
                        s.existing = existing;
                        (s.input.take(), s.interruption.take())
                    };
                    // Cancellation could not establish NoEffect. This name
                    // remains recovery-only even after every observer is lost.
                    if let Some(owner) = shared.owner.upgrade() {
                        owner
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .unresolved
                            .insert(shared.source.journal, shared.source.position);
                    }
                    drop(prepared);
                    drop(retired);
                    return Err(OrderedJournalOperationFailure::Settlement(
                        AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(cause),
                    ));
                }
            }
        }
        PreparedDurableRecordAppend::Prepared(prepared) => {
            prepared.start_ordered_journal(shared.clone());
        }
    }
    Ok(())
}
