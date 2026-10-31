//! Caller-driven publication custody, independent of an execution closure.
use super::super::{AnchoredRecordMemberLocator, AnchoredRecordState, AnchoredRecords};
use super::step_custody::*;
use super::{AnchoredRecordPublicationFailure, AnchoredRecordPublished};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use worth_proof::{LinearResource, TerminalState};

mod admission;
mod failure;
mod progression;
mod recovery;
use admission::PublicationPlan;
pub use failure::{AnchoredRecordOperationFailure, AnchoredRecordPublicationFailureCustody};
pub use recovery::{
    AnchoredRecordPublicationRecovery, AnchoredRecordPublicationResume, AnchoredRecordStoppedPoll,
    AnchoredRecordStoppedPublication,
};

worth_proof::authority_marker!(AnchoredPublicationCustodyAuthority);
enum PublicationDisposition {
    Published,
    Stopped,
    Abandoned,
}
impl TerminalState for PublicationDisposition {
    fn label(&self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::Stopped => "stopped",
            Self::Abandoned => "abandoned",
        }
    }
}
type PublicationResource = LinearResource<
    AnchoredRecordPublicationIdentity,
    PublicationDisposition,
    AnchoredPublicationCustodyAuthority,
>;

#[must_use]
pub struct AnchoredRecordPublicationOperation {
    pub(super) shared: Arc<PublicationShared>,
    custody: Option<PublicationResource>,
}

pub enum AnchoredRecordPublicationStart {
    Ready {
        operation: AnchoredRecordPublicationOperation,
        recovery: AnchoredRecordPublicationRecovery,
    },
    AlreadyPublished(AnchoredRecordPublished),
    Refused(AnchoredRecordOperationFailure),
}
pub enum AnchoredRecordPublicationStep {
    Pending(AnchoredRecordPublicationOperation),
    Ready(AnchoredRecordPublicationOperation),
    Completed(AnchoredRecordPublished),
    Failed(AnchoredRecordPublicationFailureCustody),
}

/// Descriptive facts only. An observation cannot advance or certify a selection.
#[derive(Clone)]
pub struct AnchoredRecordPublicationObservation {
    identity: AnchoredRecordPublicationIdentity,
    prefix: Vec<AnchoredRecordMemberLocator>,
    stopped: bool,
    accepted: Option<crate::physical_runtime::PhysicalMutationIdentity>,
    terminal: Option<AnchoredRecordPublished>,
    step_terminal: Option<Arc<AnchoredRecordStepTerminal>>,
}
impl AnchoredRecordPublicationObservation {
    pub fn identity(&self) -> AnchoredRecordPublicationIdentity {
        self.identity
    }
    pub fn completed_prefix(&self) -> &[AnchoredRecordMemberLocator] {
        &self.prefix
    }
    pub fn stopped(&self) -> bool {
        self.stopped
    }
    pub fn accepted_mutation(&self) -> Option<crate::physical_runtime::PhysicalMutationIdentity> {
        self.accepted
    }
    pub fn terminal(&self) -> Option<&AnchoredRecordPublished> {
        self.terminal.as_ref()
    }
    pub fn step_terminal(&self) -> Option<&Arc<AnchoredRecordStepTerminal>> {
        self.step_terminal.as_ref()
    }
}

pub(in crate::physical_runtime) struct PublicationShared {
    pub(super) identity: AnchoredRecordPublicationIdentity,
    pub(super) owner: Weak<Mutex<AnchoredRecordState>>,
    pub(super) state: Mutex<PublicationState>,
}
pub(super) struct PublicationState {
    plan: Option<PublicationPlan>,
    final_selection: Option<admission::FinalSelection>,
    recovery_alive: bool,
    pub(super) prefix: Vec<AnchoredRecordMemberLocator>,
    pub(super) written_batches: usize,
    pub(super) accepted: Option<Arc<AcceptedStep>>,
    pub(super) contents: Vec<([u8; 32], u64)>,
    pub(super) kind: AnchoredRecordStepKind,
    pub(super) last_terminal: Option<Arc<AnchoredRecordStepTerminal>>,
    pub(super) terminal: Option<AnchoredRecordPublished>,
    pub(super) stopped: bool,
    pub(super) live: bool,
    pub(super) interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    returned_custody: Option<PublicationResource>,
}
impl PublicationShared {
    pub(super) fn state(&self) -> MutexGuard<'_, PublicationState> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub(super) fn observe(&self) -> AnchoredRecordPublicationObservation {
        let state = self.state();
        AnchoredRecordPublicationObservation {
            identity: self.identity,
            prefix: state.prefix.clone(),
            stopped: state.stopped,
            accepted: state.accepted.as_ref().map(|step| step.mutation_identity()),
            terminal: state.terminal.clone(),
            step_terminal: state
                .accepted
                .as_ref()
                .and_then(|step| step.observed_terminal())
                .or_else(|| state.last_terminal.clone()),
        }
    }
    pub(super) fn require_receiver(
        &self,
        records: &AnchoredRecords<'_>,
    ) -> Result<(), AnchoredRecordOperationFailure> {
        let owner = records.runtime.anchored_publication_owner();
        if self.identity.runtime != records.runtime.runtime_identity()
            || !Weak::ptr_eq(&self.owner, &Arc::downgrade(&owner))
        {
            return Err(AnchoredRecordOperationFailure::ForeignReceiver);
        }
        let state = owner.lock().unwrap_or_else(|p| p.into_inner());
        if !state
            .active
            .as_ref()
            .is_some_and(|active| std::ptr::eq(active.as_ptr(), self))
        {
            return Err(AnchoredRecordOperationFailure::PublicationOwnerReleased);
        }
        Ok(())
    }
    pub(super) fn stop(&self) {
        let (interruption, plan, unused_selection) = {
            let mut state = self.state();
            state.stopped = true;
            let accepted_anchor =
                state.accepted.is_some() && state.kind == AnchoredRecordStepKind::Anchor;
            let unused_selection = if accepted_anchor {
                None
            } else {
                state.final_selection.take()
            };
            (
                state.interruption.take(),
                state.plan.take(),
                unused_selection,
            )
        };
        // The accepted batch owns its actual bytes and original guards.
        // Remaining inputs are no longer useful and are released outside locks.
        drop(plan);
        drop(unused_selection);
        drop(interruption);
    }
}
impl AnchoredRecordPublicationOperation {
    /// Owned inspection snapshot; cloning the completed prefix costs O(members).
    pub fn observe(&self) -> AnchoredRecordPublicationObservation {
        self.shared.observe()
    }
    /// Permanently removes advancement; it does not roll back an accepted step.
    pub fn stop(mut self) -> AnchoredRecordStoppedPublication {
        self.shared.stop();
        AnchoredRecordStoppedPublication {
            shared: self.shared.clone(),
            custody: self.custody.take(),
        }
    }
}
impl Drop for AnchoredRecordPublicationOperation {
    fn drop(&mut self) {
        if let Some(resource) = self.custody.take() {
            if std::thread::panicking() {
                self.shared.stop();
            }
            let abandoned = {
                let mut state = self.shared.state();
                state.returned_custody = Some(resource);
                !state.recovery_alive
            };
            if abandoned {
                self.shared.stop();
            }
        }
    }
}
impl Drop for PublicationShared {
    fn drop(&mut self) {
        if let Some(resource) = self
            .state
            .get_mut()
            .unwrap_or_else(|p| p.into_inner())
            .returned_custody
            .take()
        {
            let _receipt = resource.terminate(PublicationDisposition::Abandoned);
        }
    }
}
