//! Uninspected queue claims carry identity, never a version or Ready pin.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkClaim;
pub(in super::super) enum MemberInput<Ready> {
    Queued {
        claim: RequiredWorkClaim,
        ready: Option<Ready>,
    },
    Selected(Ready),
}
impl<Ready> MemberInput<Ready> {
    pub(in super::super) fn selected(&self) -> &Ready {
        match self {
            Self::Selected(ready) => ready,
            Self::Queued { ready, .. } => ready
                .as_ref()
                .expect("inspection must bind the claim first"),
        }
    }
    /// An acknowledged queue claim cannot be selected again. Keep only the
    /// native pin, which a later Upstream may inspect without queue custody.
    pub(in super::super) fn finish_claim(&mut self) {
        if let Self::Queued { ready, .. } = self {
            let ready = ready.take().expect("a Current claim owns its native pin");
            *self = Self::Selected(ready);
        }
    }
    pub(in super::super) fn rebind(&mut self, next: Ready) {
        match self {
            Self::Selected(ready) => *ready = next,
            Self::Queued { ready, .. } => *ready = Some(next),
        }
    }
}
impl RequiredMember {
    pub(in super::super) fn key(&self) -> &WorthQueryOutputDemandKey {
        match &self.ready {
            MemberInput::Queued { claim, .. } => claim.key(),
            MemberInput::Selected(ready) => ready.key(),
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
