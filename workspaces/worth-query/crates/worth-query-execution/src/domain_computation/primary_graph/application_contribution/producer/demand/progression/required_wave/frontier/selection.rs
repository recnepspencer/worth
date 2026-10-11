//! Native selections and queue claims keep their own admission payer.
//! A checkpoint binding retains the sealed, paid inspection's provenance.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkClaim;
pub(in super::super) enum MemberInput<Ready> {
    /// The popped claim pays its registry selection at first inspection.
    Queued {
        claim: RequiredWorkClaim,
        ready: Option<ReadyBinding<Ready>>,
    },
    Selected(ReadyBinding<Ready>),
}
pub(in super::super) enum ReadyBinding<Ready> {
    /// Caller admission, native Upstream certification, or held-resume inspection
    /// issued this pin; a checkpoint driver must use the paid binding instead.
    Native(Ready),
    Checkpoint(CheckpointReady<Ready>),
}
/// Only consuming PaidCheckpointReady constructs this private payload.
pub(in super::super) struct CheckpointReady<Ready> {
    ready: Ready,
}
impl<Ready> ReadyBinding<Ready> {
    fn ready(&self) -> &Ready {
        match self {
            Self::Native(ready) => ready,
            Self::Checkpoint(paid) => &paid.ready,
        }
    }
}
impl<Ready> MemberInput<Ready> {
    /// The actual owner that issued this native pin has paid for its selection.
    pub(in super::super) fn native(ready: Ready) -> Self {
        Self::Selected(ReadyBinding::Native(ready))
    }
    pub(in super::super) fn selected(&self) -> &Ready {
        self.bound_ready()
            .expect("inspection must bind the claim first")
    }
    pub(super) fn bound_ready(&self) -> Option<&Ready> {
        match self {
            Self::Selected(ready) => Some(ready.ready()),
            Self::Queued { ready, .. } => ready.as_ref().map(ReadyBinding::ready),
        }
    }
    pub(super) fn claim(&self) -> Option<&RequiredWorkClaim> {
        match self {
            Self::Queued { claim, .. } => Some(claim),
            Self::Selected(_) => None,
        }
    }
    pub(in super::super) fn claim_mut(&mut self) -> Option<&mut RequiredWorkClaim> {
        match self {
            Self::Queued { claim, .. } => Some(claim),
            Self::Selected(_) => None,
        }
    }
    pub(in super::super) fn finish_claim(&mut self) {
        if let Self::Queued { ready, .. } = self {
            let ready = ready.take().expect("a Current claim owns its pin");
            *self = Self::Selected(ready);
        }
    }
    pub(in super::super) fn clear_claim_binding(&mut self) {
        if let Self::Queued { ready, .. } = self {
            *ready = None;
        }
    }
    pub(in super::super) fn bind_inspected_native(&mut self, next: Ready) {
        self.rebind_native(next);
    }
    pub(super) fn rebind_native(&mut self, next: Ready) {
        self.bind(ReadyBinding::Native(next));
    }
    fn bind(&mut self, next: ReadyBinding<Ready>) {
        match self {
            Self::Selected(ready) => *ready = next,
            Self::Queued { ready, .. } => *ready = Some(next),
        }
    }
}
impl MemberInput<SelectedReadyReadmission> {
    pub(in super::super) fn rebind_checkpoint(
        &mut self,
        paid: super::super::super::super::required_continuations::PaidCheckpointReady,
    ) {
        self.bind(ReadyBinding::Checkpoint(CheckpointReady {
            ready: paid.into_readmission(),
        }));
    }
}
impl RequiredMember {
    pub(in super::super) fn key(&self) -> &WorthQueryOutputDemandKey {
        match self.ready.claim() {
            Some(claim) => claim.key(),
            None => self.ready.selected().key(),
        }
    }
}
impl RequiredFrontier {
    pub(in super::super) fn insert_claim(
        &mut self,
        claim: RequiredWorkClaim,
        queued: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.reserve_slot(admission)?;
        self.members.push(RequiredMember {
            ready: MemberInput::Queued { claim, ready: None },
            caller: false,
            queued: Some(queued),
            caller_successor: false,
            resolved_successor: false,
            contacts: 0,
            state: MemberState::Inspect,
            dependency_schedule: DependencySchedule::OwnSelection,
        });
        Ok(())
    }
}
