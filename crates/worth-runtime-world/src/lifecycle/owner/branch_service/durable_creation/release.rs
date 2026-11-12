//! Owned, never-submitted bindings prove only component no-effect, not World no-effect.
use super::*;
use crate::branch::{RelationalBranchCreationPlan as Native, SignalBranchCreationPlan as Signal};
use crate::durability::*;
use worth_relational::facade::branch::{
    AdmittedRelationalForkSourceBasis, RelationalForkTargetReservation,
};
#[derive(Debug)]
pub(super) struct UnsubmittedCreationOwners {
    subject: WorldDurableArtifact,
    _native_source: Option<AdmittedRelationalForkSourceBasis>,
    _native_target: Option<RelationalForkTargetReservation>,
    _signal_binding: Option<crate::lifecycle::PreparedWorldSignalFork>,
}
impl worth_proof::AuthorityMarker for UnsubmittedCreationOwners {}
impl WorldDurableCreationCustody {
    pub(super) fn prepare_unsubmitted_release(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        use RuntimeWorldBranchAdmissionDenial as Denial;
        // An existing target reservation is move-only owner custody. A submitted
        // fork consumes it, so it cannot satisfy these inputs on any later retry.
        let native_unsubmitted = match self.plan.relational() {
            Native::ReuseExact => self.native_original.is_none() && self.native_target.is_none(),
            Native::ForkExact { .. } => {
                self.native_original.is_none()
                    && self.native_source.is_some()
                    && self.native_target.is_some()
            }
        };
        let signal_unsubmitted = self.signal_task.is_none()
            && match self.plan.signal() {
                Signal::ReuseExact => self.signal_prepared.is_none(),
                Signal::ForkExact { .. } => self.signal_prepared.is_some(),
            };
        if !native_unsubmitted || !signal_unsubmitted {
            return Ok(());
        }
        let WorldRecord::CreationIntent(intent) = self.subject.record() else {
            unreachable!("original intent")
        };
        let release = WorldDurableArtifact::from_input(
            WorldArtifactInput::CreationReleased {
                association: &intent.association,
                custody: intent.custody,
            },
            request,
        )
        .map_err(|_| Denial::CapacityExhausted)?;
        let original = UnsubmittedCreationOwners {
            subject: self.subject.clone(),
            _native_source: self.native_source.take(),
            _native_target: self.native_target.take(),
            _signal_binding: self.signal_prepared.take(),
        };
        self.unsubmitted = Some(worth_proof::ProvenNoEffectPublication::from_owner(original));
        self.release_subject = Some(release);
        self.phase = CreationPhase::Releasing;
        Ok(())
    }
    pub(super) fn drive_unsubmitted_release(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        use RuntimeWorldBranchAdmissionDenial as Denial;
        let owners = self
            .unsubmitted
            .as_ref()
            .expect("same actual unsubmitted bindings");
        assert!(
            owners.original().subject.same_subject(&self.subject),
            "owner no-effect proof is original-affine"
        );
        let subject = self
            .release_subject
            .as_ref()
            .expect("same original release");
        match self.binding.publish(subject, request) {
            Ok(actual) => {
                let permit = self.permit.as_mut().expect("same source permit");
                permit.acknowledge_disposition(actual);
                let proof = self
                    .permit
                    .take()
                    .expect("original source")
                    .release_recorded();
                self.released =
                    Some(crate::lifecycle::RuntimeWorldBranchCreationRelease::from_original(proof));
                self.phase = CreationPhase::Released;
                Ok(())
            }
            Err(
                WorldDurablePublicationStop::Pending
                | WorldDurablePublicationStop::Busy
                | WorldDurablePublicationStop::Refused(_),
            ) => Err(Denial::OwnerUnavailable),
            Err(
                WorldDurablePublicationStop::Unresolved
                | WorldDurablePublicationStop::EvidenceRequired,
            ) => {
                self.phase = CreationPhase::ReopenRequired;
                Err(Denial::OwnerUnavailable)
            }
        }
    }
}
