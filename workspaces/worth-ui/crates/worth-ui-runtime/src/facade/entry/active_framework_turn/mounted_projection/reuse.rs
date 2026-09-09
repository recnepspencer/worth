use super::WorthUiActiveFrameworkTurnExecution;

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(super) fn reuse_contract(
        &self,
        request: &crate::mounting::UiMountedFrameRequest,
        lanes: crate::mounting::UiMountedLaneAssembly,
        allocation_truth_revision: u64,
    ) -> crate::mounting::UiMountedFrameReuseContract {
        self.mounted
            .seal_frame_reuse_contract(crate::mounting::UiMountedFrameReuseExternalBasis {
                generation: self.generation_identity.clone(),
                host_session: self.host_session_identity.as_u64(),
                execution: crate::mounting::UiMountedFrameExecutionPosture::ActiveFrame {
                    frame_epoch: self.execution.active_frame_epoch().as_u64(),
                },
                plan_digest: self.execution.active_plan_digest(),
                allocation_truth_revision,
                request: request.reuse_identity(),
                lanes,
                protocol: self.host_protocol,
                capability_generation: self.host_capability_generation,
                capability_profile_digest: self.host_capability_profile_digest,
                visual_overlay_revision: request.visual_overlay_revision(),
                pointer_affordance: crate::runtime::pointer_affordance::UiPointerAffordanceReuseBasis::from_snapshot(
                    self.pointer_affordance_snapshot.as_ref(), |surface| request.includes_surface(surface),
                ),
            })
    }
}
