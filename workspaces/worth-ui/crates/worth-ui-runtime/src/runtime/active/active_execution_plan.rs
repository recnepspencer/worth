use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiActiveExecutionPlan {
    digest: WorthUiActiveExecutionPlanDigest,
    bundle: Rc<super::WorthUiSealedExecutionPlanBundle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiActiveExecutionPlanDigest {
    value: u64,
}

impl WorthUiActiveExecutionPlan {
    pub(crate) fn digest(&self) -> WorthUiActiveExecutionPlanDigest {
        self.digest
    }

    pub(crate) fn cross_lane_receipt(&self) -> super::WorthUiCrossLaneBundleReceipt {
        self.bundle.cross_lane_receipt()
    }

    pub(crate) fn mounted_projection_plan_index(&self, provenance: u64) -> Result<Option<u32>, ()> {
        self.bundle.mounted_projection_plan_index(provenance)
    }

    pub(crate) fn mounted_projection_ordinary_meaning(
        &self,
        plan_index: u32,
    ) -> Option<
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    > {
        self.bundle.mounted_projection_ordinary_meaning(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_with_identity(
        &self,
        plan_index: u32,
    ) -> Option<(
        String,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.bundle
            .mounted_projection_ordinary_meaning_with_identity(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_for_identity(
        &self,
        identity: &str,
    ) -> Option<(
        u32,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.bundle
            .mounted_projection_ordinary_meaning_for_identity(identity)
    }

    pub(crate) fn mounted_projection_theme_token(
        &self,
        token_id: &crate::capability::ThemeTokenId,
    ) -> Result<
        Option<(
            u32,
            std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
        )>,
        (),
    > {
        self.bundle.mounted_projection_theme_token(token_id)
    }

    pub(crate) fn classify_candidate(
        &self,
        candidate: &super::WorthUiSealedExecutionPlanBundle,
    ) -> crate::runtime::WorthUiExecutablePlanDecision {
        self.bundle.classify_candidate(candidate)
    }

    pub(crate) fn query_succession_changes(
        &self,
        candidate: &super::WorthUiSealedExecutionPlanBundle,
    ) -> Vec<worth_ui_query_binding::WorthUiQueryBindingSuccessionChange> {
        self.bundle.query_succession_changes(candidate)
    }

    pub(crate) fn from_lowered_bundle(bundle: super::WorthUiSealedExecutionPlanBundle) -> Self {
        let candidate_digest = bundle.digest();
        Self {
            digest: WorthUiActiveExecutionPlanDigest {
                value: candidate_digest.raw(),
            },
            bundle: Rc::new(bundle),
        }
    }

    pub(crate) fn generation_identity(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        self.bundle.generation_identity()
    }

    pub(crate) fn ordinary_availability(&self) -> crate::runtime::WorthUiOrdinaryPlanAvailability {
        self.bundle.ordinary_availability()
    }

    pub(crate) fn virtualized_availability(
        &self,
    ) -> crate::runtime::WorthUiVirtualizedPlanAvailability {
        self.bundle.virtualized_availability()
    }

    pub(crate) fn query_fact_link_for_plan_index(
        &self,
        plan_index: u32,
    ) -> Option<crate::runtime::WorthUiQuerySettledFactLink> {
        self.bundle.query_fact_link_for_plan_index(plan_index)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn query_fact_link_for_binding_id(
        &self,
        binding_id: &crate::capability::ViewBindingId,
    ) -> Option<crate::runtime::WorthUiQueryLaneFactLink> {
        self.bundle.query_fact_link_for_binding_id(binding_id)
    }

    pub(crate) fn query_plan_state_observation(
        &self,
        binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    ) -> super::WorthUiActiveQueryPlanObservation {
        self.bundle.query_plan_state_observation(binding)
    }

    pub(crate) fn canvas_spatial_availability(
        &self,
    ) -> crate::runtime::WorthUiCanvasSpatialPlanAvailability {
        self.bundle.canvas_spatial_availability()
    }

    pub(crate) fn first_canvas_spatial_handle(&self) -> Option<crate::runtime::WorthUiLaneHandle> {
        self.bundle.first_canvas_spatial_handle()
    }

    pub(crate) fn execute_canvas_spatial(
        &self,
        target: crate::runtime::WorthUiCanvasSpatialFrameTarget,
    ) -> Result<
        crate::runtime::WorthUiCanvasSpatialFrameReceipt,
        crate::runtime::WorthUiCanvasSpatialFrameDenial,
    > {
        self.bundle.execute_canvas_spatial(target)
    }

    pub(crate) fn canvas_spatial_summary(
        &self,
        handle: crate::runtime::WorthUiLaneHandle,
    ) -> Result<
        crate::runtime::WorthUiCanvasSpatialTargetSummary,
        crate::runtime::WorthUiCanvasSpatialInspectionDenial,
    > {
        self.bundle.canvas_spatial_summary(handle)
    }

    pub(crate) fn realtime_availability(&self) -> crate::runtime::WorthUiRealtimePlanAvailability {
        self.bundle.realtime_availability()
    }

    pub(crate) fn first_realtime_handle(
        &self,
    ) -> Option<crate::runtime::WorthUiRendererSurfaceHandle> {
        self.bundle.first_realtime_handle()
    }

    pub(crate) fn execute_realtime(
        &self,
        target: crate::runtime::WorthUiRealtimeFrameTarget,
    ) -> Result<
        crate::runtime::WorthUiRealtimeFrameReceipt,
        crate::runtime::WorthUiRealtimeFrameDenial,
    > {
        self.bundle.execute_realtime(target)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn realtime_summary(
        &self,
        handle: crate::runtime::WorthUiRendererSurfaceHandle,
    ) -> Result<
        crate::runtime::WorthUiRealtimeTargetSummary,
        crate::runtime::WorthUiRealtimeInspectionDenial,
    > {
        self.bundle.realtime_summary(handle)
    }

    pub(crate) fn execute_virtualized(
        &self,
        query_binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
        target: crate::runtime::WorthUiVirtualizedDataFrameTarget,
    ) -> Result<
        crate::runtime::WorthUiVirtualizedDataFrameReceipt,
        crate::runtime::WorthUiVirtualizedDataFrameDenial,
    > {
        self.bundle.execute_virtualized(query_binding, target)
    }

    pub(crate) fn virtualized_summary(
        &self,
        query_binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
        request: crate::runtime::WorthUiVirtualizedPlanSummaryRequest,
    ) -> Result<
        crate::runtime::WorthUiVirtualizedPlanSummary,
        crate::runtime::WorthUiVirtualizedPlanSummaryDenial,
    > {
        self.bundle.virtualized_summary(query_binding, request)
    }

    pub(crate) fn handle_arena_identity(&self) -> crate::runtime::WorthUiHandleArenaIdentity {
        self.bundle.handle_arena_identity()
    }

    pub(crate) fn shares_lowering_identity_with(
        &self,
        identity: &crate::runtime::planning::WorthUiExecutionPlanLoweringIdentity,
    ) -> bool {
        self.bundle.shares_lowering_identity_with(identity)
    }

    pub(crate) fn execute_ordinary(
        &self,
        target: crate::runtime::WorthUiOrdinaryFrameTarget,
    ) -> Result<
        crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
        crate::runtime::WorthUiOrdinaryLaneFrameDenial,
    > {
        self.bundle.execute_ordinary(target)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn ordinary_summary(
        &self,
        request: crate::runtime::WorthUiOrdinaryPlanSummaryRequest,
    ) -> Result<
        crate::runtime::WorthUiOrdinaryPlanSummary,
        crate::runtime::WorthUiOrdinaryPlanSummaryDenial,
    > {
        self.bundle.ordinary_summary(request)
    }

    pub(super) fn predecessor_region_proof(
        &self,
        active_artifact_digest: u64,
        authority: &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
    ) -> Result<
        crate::runtime::planning::plan_topology::WorthUiPredecessorRegionProof,
        crate::runtime::planning::plan_topology::WorthUiPredecessorRegionProofDenial,
    > {
        crate::runtime::planning::plan_topology::WorthUiPredecessorRegionProof::from_active_plan(
            Rc::clone(&self.bundle),
            self.digest.as_u64(),
            active_artifact_digest,
            authority,
        )
    }

    #[cfg(test)]
    pub(crate) fn exact_plan(&self) -> &crate::runtime::WorthUiExecutionPlan {
        self.bundle.execution_plan()
    }
}

impl WorthUiActiveExecutionPlanDigest {
    pub(crate) fn as_u64(self) -> u64 {
        self.value
    }
}
