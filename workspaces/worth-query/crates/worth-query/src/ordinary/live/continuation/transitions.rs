use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary::live::WorthQueryManagedLiveHandle;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryRuntimeBasisPostureKind, WorthQueryOrdinaryRuntimePostureKind,
};
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryWorkspace};

use super::{
    WorthQueryManagedLiveCheckpointCompletion, WorthQueryManagedLiveCheckpointOutcome,
    WorthQueryManagedLiveCheckpointReceipt, WorthQueryManagedLiveCheckpointStop,
    WorthQueryManagedLiveContinuation, WorthQueryManagedLiveResumeCompletion,
    WorthQueryManagedLiveResumeOutcome, WorthQueryManagedLiveResumeReceipt,
    WorthQueryManagedLiveResumeStop, WorthQueryManagedLiveResumeStopKind,
};

impl WorthQueryManagedLiveHandle {
    pub fn checkpoint(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryManagedLiveCheckpointOutcome, IdentityDerivationDenial> {
        let observation = match self.observe(workspace) {
            Ok(observation) => observation,
            Err(error) => {
                return Ok(WorthQueryManagedLiveCheckpointOutcome::Stopped(
                    WorthQueryManagedLiveCheckpointStop::new(self, error),
                ));
            }
        };
        let checkpoint = WorthQueryManagedLiveCheckpointReceipt::new(
            observation.resource_name(),
            observation.installation_identity(),
            observation.basis_binding_identity(),
            observation.pending_delivery_batch_count(),
            observation.last_delivery_sequence(),
        )?;
        let (view, capability, projection_binding) = self.into_resource_parts();
        Ok(WorthQueryManagedLiveCheckpointOutcome::Checkpointed(
            WorthQueryManagedLiveCheckpointCompletion::new(WorthQueryManagedLiveContinuation::new(
                view,
                capability,
                checkpoint,
                projection_binding,
            )),
        ))
    }
}

impl WorthQueryManagedLiveContinuation {
    pub fn resume(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryManagedLiveResumeOutcome, IdentityDerivationDenial> {
        if let Err(error) =
            workspace.admit_managed_live_capability(self.workspace_capability(), self.view().name())
        {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::ForeignWorkspace,
                Some(error),
            ));
        }
        let observation =
            match workspace.observe_managed_live_view(self.view(), self.workspace_capability()) {
                Ok(observation) => observation,
                Err(error) => {
                    return Ok(stopped(
                        self,
                        WorthQueryManagedLiveResumeStopKind::MissingResource,
                        Some(error),
                    ));
                }
            };
        if observation.authority_lane() != WorthQueryAuthorityLane::AuthoritativeTruth {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::PreviewIsolation,
                None,
            ));
        }
        if observation.installation_identity() != self.checkpoint().installation_identity()
            || observation.basis_binding_identity() != self.checkpoint().basis_binding_identity()
        {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::ContinuationIdentityMismatch,
                None,
            ));
        }
        if observation.runtime_posture().basis_posture()
            != WorthQueryOrdinaryRuntimeBasisPostureKind::Stable
        {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::StaleBasis,
                None,
            ));
        }
        if observation.runtime_posture().remask_posture().is_some() {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::AuthorityRebindRequired,
                None,
            ));
        }
        if !runtime_posture_can_resume(observation.runtime_posture().kind()) {
            return Ok(stopped(
                self,
                WorthQueryManagedLiveResumeStopKind::RuntimeStateUnavailable,
                None,
            ));
        }

        let receipt = WorthQueryManagedLiveResumeReceipt::new(
            self.checkpoint(),
            observation.last_delivery_sequence(),
            observation.pending_delivery_batch_count(),
        )?;
        let (view, capability, projection_binding) = self.into_resource_parts();
        Ok(WorthQueryManagedLiveResumeOutcome::Resumed(
            WorthQueryManagedLiveResumeCompletion::new(
                WorthQueryManagedLiveHandle::new(view, capability, projection_binding),
                receipt,
            ),
        ))
    }
}

fn runtime_posture_can_resume(kind: WorthQueryOrdinaryRuntimePostureKind) -> bool {
    matches!(
        kind,
        WorthQueryOrdinaryRuntimePostureKind::Current
            | WorthQueryOrdinaryRuntimePostureKind::Pending
            | WorthQueryOrdinaryRuntimePostureKind::Retried
            | WorthQueryOrdinaryRuntimePostureKind::Revalidating
    )
}

fn stopped(
    continuation: WorthQueryManagedLiveContinuation,
    kind: WorthQueryManagedLiveResumeStopKind,
    runtime_error: Option<crate::runtime::WorthQueryRuntimeError>,
) -> WorthQueryManagedLiveResumeOutcome {
    WorthQueryManagedLiveResumeOutcome::Stopped(WorthQueryManagedLiveResumeStop::new(
        continuation,
        kind,
        runtime_error,
    ))
}
