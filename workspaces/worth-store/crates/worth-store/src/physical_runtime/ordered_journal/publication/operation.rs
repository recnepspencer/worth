//! Move-only journal progression, retained independently of an execution result.
use super::{outcome::*, recovery::*, *};
use crate::physical_runtime::{
    PhysicalMutationAttempt, PhysicalMutationIdentity, PhysicalMutationTerminalFact,
};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use worth_proof::{LinearResource, TerminalState};
pub(super) enum Disposition {
    Published,
    Stopped,
    Abandoned,
}
impl TerminalState for Disposition {
    fn label(&self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::Stopped => "stopped",
            Self::Abandoned => "abandoned",
        }
    }
}
pub(super) type Resource = LinearResource<
    OrderedJournalPublicationSource,
    Disposition,
    OrderedJournalPublicationAuthority,
>;
#[must_use]
pub struct OrderedJournalPublicationOperation {
    pub(super) shared: Arc<JournalPublicationShared>,
    pub(super) custody: Option<Resource>,
}
pub enum OrderedJournalPublicationStart {
    Ready {
        operation: OrderedJournalPublicationOperation,
        recovery: OrderedJournalPublicationRecovery,
    },
    AlreadyCommitted(OrderedJournalCommitted),
    Refused(OrderedJournalOperationFailure),
}
pub enum OrderedJournalPublicationStep {
    Pending(OrderedJournalPublicationOperation),
    Completed(OrderedJournalCommitted),
    Failed(OrderedJournalPublicationFailureCustody),
}
pub(in crate::physical_runtime) struct JournalPublicationShared {
    pub(super) source: OrderedJournalPublicationSource,
    pub(super) owner: Weak<Mutex<OrderedJournalState>>,
    pub(super) state: Mutex<JournalProgress>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum JournalTerminalVisibility {
    AwaitingBasePublication,
    Published,
    WorkerLost,
}
pub(super) struct JournalProgress {
    pub(super) plan: Option<crate::physical_runtime::record_serving::AnchoredRecordAppend>,
    pub(super) input: Option<worth_execution::ExecutionMemoryReservation>,
    pub(super) interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    pub(super) preparing: bool,
    pub(super) stopped: bool,
    pub(super) live: bool,
    pub(super) recovery_alive: bool,
    pub(super) endpoint_live: bool,
    pub(super) returned: Option<Resource>,
    pub(super) mutation: Option<PhysicalMutationIdentity>,
    pub(super) attempt: Weak<PhysicalMutationAttempt>,
    pub(super) existing:
        Option<Arc<crate::physical_runtime::durability::ExistingPhysicalMutationObservation>>,
    pub(super) refusal: Option<crate::physical_runtime::PhysicalPreSealCancellationDenial>,
    pub(super) unresolved_duplicate: bool,
    pub(super) fact: Option<PhysicalMutationTerminalFact>,
    pub(super) settling: bool,
    pub(super) terminal: Option<Arc<OrderedJournalPublicationTerminal>>,
    pub(super) visibility: JournalTerminalVisibility,
}
impl JournalPublicationShared {
    pub(super) fn state(&self) -> MutexGuard<'_, JournalProgress> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub(super) fn require_receiver(
        &self,
        journal: &OrderedJournal<'_>,
    ) -> Result<(), OrderedJournalOperationFailure> {
        let owner = journal.runtime.ordered_journal_owner();
        if journal.name != self.source.journal
            || journal.runtime.runtime_identity() != self.source.runtime
            || !Weak::ptr_eq(&self.owner, &Arc::downgrade(&owner))
        {
            return Err(OrderedJournalOperationFailure::ForeignReceiver);
        }
        let inert = {
            let state = self.state();
            state.terminal.is_some()
                || (state.stopped && state.mutation.is_none() && !state.preparing)
        };
        if inert {
            return Ok(());
        }
        let state = owner.lock().unwrap_or_else(|p| p.into_inner());
        if !state
            .active
            .as_ref()
            .is_some_and(|p| std::ptr::eq(p.as_ptr(), self))
        {
            return Err(OrderedJournalOperationFailure::OwnerReleased);
        }
        Ok(())
    }
    pub(super) fn observe(&self) -> OrderedJournalPublicationObservation {
        self.observe_existing();
        let terminal = self.visible_terminal();
        let state = self.state();
        OrderedJournalPublicationObservation {
            source: self.source,
            mutation: state.mutation,
            stopped: state.stopped,
            terminal,
        }
    }
    pub(super) fn stop(&self) {
        let retired = {
            let mut s = self.state();
            s.stopped = true;
            let plan = s.plan.take();
            let view = s.interruption.take();
            let input = if s.mutation.is_none() && !s.preparing {
                s.live = false;
                s.input.take()
            } else {
                None
            };
            (plan, view, input)
        };
        drop(retired);
    }
    pub(in crate::physical_runtime) fn bind(&self, attempt: &Arc<PhysicalMutationAttempt>) {
        let mut s = self.state();
        assert_eq!(
            s.mutation,
            Some(attempt.identity()),
            "one exact journal mutation"
        );
        s.attempt = Arc::downgrade(attempt);
    }
    pub(in crate::physical_runtime) fn preserve_fact(&self, fact: &PhysicalMutationTerminalFact) {
        self.state().fact = Some(fact.clone());
    }
}
impl OrderedJournalPublicationOperation {
    pub fn observe(&self) -> OrderedJournalPublicationObservation {
        self.shared.observe()
    }
    pub fn stop(mut self) -> OrderedJournalStoppedPublication {
        self.shared.stop();
        OrderedJournalStoppedPublication {
            shared: self.shared.clone(),
            custody: self.custody.take(),
        }
    }
}
impl Drop for OrderedJournalPublicationOperation {
    fn drop(&mut self) {
        if let Some(resource) = self.custody.take() {
            if std::thread::panicking() {
                self.shared.stop();
            }
            let abandoned = {
                let mut s = self.shared.state();
                s.endpoint_live = false;
                s.returned = Some(resource);
                !s.recovery_alive
            };
            if abandoned {
                self.shared.stop();
            }
        }
    }
}
impl Drop for JournalPublicationShared {
    fn drop(&mut self) {
        if let Some(r) = self
            .state
            .get_mut()
            .unwrap_or_else(|p| p.into_inner())
            .returned
            .take()
        {
            let _ = r.terminate(Disposition::Abandoned);
        }
    }
}
impl std::fmt::Debug for OrderedJournalPublicationOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OrderedJournalPublicationOperation")
            .field(&self.observe())
            .finish()
    }
}
