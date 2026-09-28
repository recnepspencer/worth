use std::num::NonZeroU16;

#[cfg(test)]
use super::UiAllocationFrameReplacementTransition;
use super::{
    UiAdmittedAllocationStreamFrame, UiAdmittedAllocationStreamIngress,
    UiAllocationFrameDispatchDenial, UiAllocationFrameDispatcherCounters,
    UiAllocationFrameDispatcherState, UiAllocationFrameEpoch, UiAllocationFrameMailbox,
    UiAllocationFramePauseReason, UiAllocationFrameQueueDisposition, UiAllocationFrameRetryState,
    UiAllocationFrameSubmissionAssignmentBatch, UiAllocationFrameSubmissionDenial,
    UiAllocationFrameSubmissionOutcome, UiAllocationFrameTransitionOutcome,
};

use super::{
    UiAllocationFrameSourceGeneration, UiAllocationFrameSourceIdentity,
    UiAllocationFrameSourceLane, UiAllocationFrameSourceLease,
};

mod epoch_assignment;
mod replacement;
mod submission;
mod submission_transition;
pub(crate) use epoch_assignment::UiAllocationFrameEpochAssignment;
pub(crate) use submission_transition::UiAllocationFrameSubmissionTransition;

const ALLOCATION_FRAME_MAILBOX_CAPACITY: NonZeroU16 =
    NonZeroU16::new(super::ALLOCATION_FRAME_SOURCE_COUNT).unwrap();

#[derive(Clone, Copy, Debug)]
struct UiAllocationFrameCloseTrigger;

/// Unforgeable capability required to materialize a sealed dispatcher output.
#[derive(Debug)]
pub(super) struct UiAllocationFrameSealAuthority(());

#[cfg(test)]
pub(super) struct UiAllocationFrameDispatcherTestAuthority(());

impl UiAllocationFrameCloseTrigger {
    fn runtime_pump_turn() -> Self {
        Self
    }
}

/// Sole runtime authority that linearizes allocation frame lifecycle and epoch assignment.
#[derive(Debug)]
pub(crate) struct UiAllocationFrameDispatcher {
    state: UiAllocationFrameDispatcherState,
    mailbox: UiAllocationFrameMailbox,
    successor_mailbox: UiAllocationFrameMailbox,
    counters: UiAllocationFrameDispatcherCounters,
    seal_authority: UiAllocationFrameSealAuthority,
    sealed_frame: Option<UiAdmittedAllocationStreamFrame>,
    retry_state: UiAllocationFrameRetryState,
    source_registry: super::UiAllocationFrameSourceRegistry,
}

impl UiAllocationFrameDispatcher {
    pub(crate) fn launch(epoch: UiAllocationFrameEpoch) -> Self {
        Self::with_capacity(epoch, ALLOCATION_FRAME_MAILBOX_CAPACITY)
    }

    #[cfg(test)]
    fn launch_for_test(epoch: UiAllocationFrameEpoch, capacity: NonZeroU16) -> Self {
        Self::with_capacity(epoch, capacity)
    }

    #[cfg(test)]
    fn ingress_for_test(
        lease: &UiAllocationFrameSourceLease,
        identity: super::UiAllocationFrameIngressIdentity,
        source_order: super::UiAdmittedAllocationSourceOrder,
    ) -> UiAdmittedAllocationStreamIngress {
        lease.admit_ingress(
            &UiAllocationFrameDispatcherTestAuthority(()),
            identity,
            source_order,
        )
    }

    fn with_capacity(epoch: UiAllocationFrameEpoch, capacity: NonZeroU16) -> Self {
        let mailbox = UiAllocationFrameMailbox::new(capacity.get());
        Self {
            state: UiAllocationFrameDispatcherState::Open(epoch),
            counters: UiAllocationFrameDispatcherCounters::empty(mailbox.storage_posture()),
            mailbox,
            successor_mailbox: UiAllocationFrameMailbox::new(capacity.get()),
            seal_authority: UiAllocationFrameSealAuthority(()),
            sealed_frame: None,
            retry_state: UiAllocationFrameRetryState::empty(epoch),
            source_registry: super::UiAllocationFrameSourceRegistry::empty(epoch.as_u64()),
        }
    }

    pub(crate) fn state(&self) -> UiAllocationFrameDispatcherState {
        self.state
    }

    pub(crate) fn counters(&self) -> UiAllocationFrameDispatcherCounters {
        self.counters
    }

    fn begin_close_for_runtime_pump(
        &mut self,
        _trigger: UiAllocationFrameCloseTrigger,
    ) -> Result<(), UiAllocationFrameDispatchDenial> {
        let epoch = match self.state {
            UiAllocationFrameDispatcherState::Open(epoch) => epoch,
            UiAllocationFrameDispatcherState::Dispatched(_)
                if self.successor_mailbox.is_empty() =>
            {
                return Err(UiAllocationFrameDispatchDenial::EmptyFrame);
            }
            UiAllocationFrameDispatcherState::Paused(
                UiAllocationFramePauseReason::EpochExhausted,
            ) => return Err(UiAllocationFrameDispatchDenial::EpochExhausted),
            UiAllocationFrameDispatcherState::Paused(_)
            | UiAllocationFrameDispatcherState::Closing { .. }
            | UiAllocationFrameDispatcherState::Sealed(_)
            | UiAllocationFrameDispatcherState::Dispatched(_) => {
                return Err(UiAllocationFrameDispatchDenial::NoOpenFrame)
            }
        };
        let Some(next_epoch) = epoch.checked_next() else {
            self.state = UiAllocationFrameDispatcherState::Paused(
                UiAllocationFramePauseReason::EpochExhausted,
            );
            self.counters.record_terminal_denial();
            return Err(UiAllocationFrameDispatchDenial::EpochExhausted);
        };
        if self.mailbox.is_empty() {
            return Err(UiAllocationFrameDispatchDenial::EmptyFrame);
        }
        self.state = UiAllocationFrameDispatcherState::Closing { epoch, next_epoch };
        Ok(())
    }

    fn finish_close_for_runtime_pump(&mut self) {
        let UiAllocationFrameDispatcherState::Closing { epoch, .. } = self.state else {
            unreachable!("only the close authority may finish an active close");
        };
        self.counters.record_canonical_drain();
        let (ingress, drain_work) = self.mailbox.drain_canonical();
        self.counters
            .record_mailbox_insert(drain_work.comparisons, drain_work.canonical_writes);
        debug_assert!(!ingress.is_empty());
        let assignments = UiAllocationFrameSubmissionAssignmentBatch::linearize(
            &self.seal_authority,
            epoch,
            &ingress,
        );
        for sealed_assignment in assignments.iter(epoch, &ingress) {
            let (comparisons, writes) =
                self.retry_state.commit_sealed_assignment(sealed_assignment);
            self.counters.record_retry_ledger_work(comparisons, writes);
        }
        self.counters.record_frame();
        self.sealed_frame = Some(UiAdmittedAllocationStreamFrame::new(
            &self.seal_authority,
            epoch,
            ingress,
            assignments,
            self.counters,
        ));
        self.state = UiAllocationFrameDispatcherState::Sealed(epoch);
    }

    /// Seals one canonical frame. Only `dispatch` composes this with ordinary dispatch.
    fn seal_for_runtime_pump(
        &mut self,
        trigger: UiAllocationFrameCloseTrigger,
    ) -> Result<(), UiAllocationFrameDispatchDenial> {
        self.begin_close_for_runtime_pump(trigger)?;
        self.finish_close_for_runtime_pump();
        Ok(())
    }

    fn dispatch_sealed_frame(&mut self) -> UiAllocationFrameTransitionOutcome {
        let UiAllocationFrameDispatcherState::Sealed(epoch) = self.state else {
            return UiAllocationFrameTransitionOutcome::denied(
                &self.seal_authority,
                UiAllocationFrameDispatchDenial::NoOpenFrame,
                self.counters,
            );
        };
        let frame = self
            .sealed_frame
            .take()
            .expect("sealed lifecycle state always retains its immutable frame");
        self.state = UiAllocationFrameDispatcherState::Dispatched(epoch);
        UiAllocationFrameTransitionOutcome::dispatched(&self.seal_authority, frame)
    }

    /// The deterministic runtime-pump act: close, seal, then dispatch exactly once.
    pub(super) fn dispatch(
        &mut self,
        _turn_authority: super::framework_turn::UiAllocationFrameDispatchAuthority,
    ) -> UiAllocationFrameTransitionOutcome {
        self.perform_dispatch()
    }

    fn perform_dispatch(&mut self) -> UiAllocationFrameTransitionOutcome {
        self.activate_queued_successor();
        let trigger = UiAllocationFrameCloseTrigger::runtime_pump_turn();
        match self.seal_for_runtime_pump(trigger) {
            Ok(()) => self.dispatch_sealed_frame(),
            Err(denial) => UiAllocationFrameTransitionOutcome::denied(
                &self.seal_authority,
                denial,
                self.counters,
            ),
        }
    }

    #[cfg(test)]
    fn dispatch_for_test(&mut self) -> UiAllocationFrameTransitionOutcome {
        self.perform_dispatch()
    }

    fn activate_queued_successor(&mut self) {
        let UiAllocationFrameDispatcherState::Dispatched(epoch) = self.state else {
            return;
        };
        if self.successor_mailbox.is_empty() {
            return;
        }
        let Some(next_epoch) = epoch.checked_next() else {
            self.state = UiAllocationFrameDispatcherState::Paused(
                UiAllocationFramePauseReason::EpochExhausted,
            );
            return;
        };
        std::mem::swap(&mut self.mailbox, &mut self.successor_mailbox);
        let (comparisons, writes) = self.retry_state.begin_epoch(next_epoch);
        self.counters.record_retry_ledger_work(comparisons, writes);
        self.state = UiAllocationFrameDispatcherState::Open(next_epoch);
    }

    #[cfg(test)]
    pub(super) fn pause_for_replacement(&mut self) -> UiAllocationFrameReplacementTransition {
        let Some(successor_epoch) = self.replacement_successor_epoch() else {
            self.counters.record_terminal_denial();
            let queue_disposition = self.pause(UiAllocationFramePauseReason::EpochExhausted);
            return UiAllocationFrameReplacementTransition::denied(
                &self.seal_authority,
                queue_disposition,
                UiAllocationFrameDispatchDenial::EpochExhausted,
                self.retry_state.clone(),
            );
        };
        let queue_disposition = self.pause(UiAllocationFramePauseReason::Replacement);
        UiAllocationFrameReplacementTransition::paused(
            &self.seal_authority,
            queue_disposition,
            UiAllocationFrameEpochAssignment::from_linearization(successor_epoch),
            self.retry_state.clone(),
        )
    }

    pub(super) fn shutdown(&mut self) -> UiAllocationFrameQueueDisposition {
        self.pause(UiAllocationFramePauseReason::Shutdown)
    }

    #[cfg(test)]
    pub(super) fn retire_source(
        &mut self,
        retirement: UiAllocationFrameSourceLease,
    ) -> super::UiAllocationFrameSourceRetirementOutcome {
        if let Err(denial) = self.source_registry.validate_retirement(&retirement) {
            return super::UiAllocationFrameSourceRetirementOutcome::denied_from_registry(
                denial,
                self.counters,
                retirement,
            );
        }
        if let Err(denial) = self.retry_state.retire_source(&retirement) {
            return super::UiAllocationFrameSourceRetirementOutcome::denied_from_registry(
                denial,
                self.counters,
                retirement,
            );
        }
        self.source_registry.retire_validated(&retirement);
        super::UiAllocationFrameSourceRetirementOutcome::retired(self.counters)
    }

    pub(super) fn admit_source_generation(
        &mut self,
        lane: UiAllocationFrameSourceLane,
        identity: UiAllocationFrameSourceIdentity,
        generation: UiAllocationFrameSourceGeneration,
    ) -> Result<UiAllocationFrameSourceLease, super::UiAllocationFrameSourceAdmissionDenial> {
        self.source_registry.admit(lane, identity, generation)
    }

    pub(super) fn advance_source_generation(
        &mut self,
        lease: &UiAllocationFrameSourceLease,
        generation: UiAllocationFrameSourceGeneration,
    ) -> Result<UiAllocationFrameSourceLease, super::UiAllocationFrameSourceAdmissionDenial> {
        self.source_registry.advance_generation(lease, generation)
    }

    fn pause(&mut self, reason: UiAllocationFramePauseReason) -> UiAllocationFrameQueueDisposition {
        self.state = UiAllocationFrameDispatcherState::Paused(reason);
        self.counters.record_canonical_drain();
        let (successor_ingress, successor_drain_work) = self.successor_mailbox.drain_canonical();
        self.counters.record_mailbox_insert(
            successor_drain_work.comparisons,
            successor_drain_work.canonical_writes,
        );
        let (successor_comparisons, successor_writes) =
            self.retry_state.discard_pending(successor_ingress.view());
        self.counters
            .record_retry_ledger_work(successor_comparisons, successor_writes);
        if let Some(frame) = self.sealed_frame.take() {
            return UiAllocationFrameQueueDisposition::sealed(
                &self.seal_authority,
                reason,
                frame,
                successor_ingress,
                self.counters,
            );
        }
        let (ingress, drain_work) = self.mailbox.drain_canonical();
        self.counters
            .record_mailbox_insert(drain_work.comparisons, drain_work.canonical_writes);
        let (ingress_comparisons, ingress_writes) =
            self.retry_state.discard_pending(ingress.view());
        self.counters
            .record_retry_ledger_work(ingress_comparisons, ingress_writes);
        UiAllocationFrameQueueDisposition::disposed(
            &self.seal_authority,
            reason,
            ingress,
            successor_ingress,
            self.counters,
        )
    }
}

#[cfg(test)]
mod tests;
