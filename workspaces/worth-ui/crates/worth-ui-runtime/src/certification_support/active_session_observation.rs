use crate::facade::{WorthUiActiveApplicationSession, WorthUiHostMeasurementCapability};
use crate::fact_contract::UiProducedFact;
use crate::graph::{UiGraphFactLookupDenial, UiGraphFactLookupReceipt};
use crate::runtime::{
    WorthUiActiveRuntimeObservation, WorthUiCanvasSpatialInspectionDenial,
    WorthUiCanvasSpatialPlanAvailability, WorthUiCanvasSpatialTargetSummary, WorthUiLaneHandle,
    WorthUiOrdinaryPlanAvailability, WorthUiOrdinaryPlanSummary, WorthUiOrdinaryPlanSummaryDenial,
    WorthUiOrdinaryPlanSummaryRequest, WorthUiQueryLaneFactLink, WorthUiRealtimeInspectionDenial,
    WorthUiRealtimePlanAvailability, WorthUiRealtimeTargetSummary, WorthUiRendererSurfaceHandle,
    WorthUiStateQueryResidueScan, WorthUiVirtualizedPlanAvailability,
    WorthUiVirtualizedPlanSummary, WorthUiVirtualizedPlanSummaryDenial,
    WorthUiVirtualizedPlanSummaryRequest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiSemanticTextCertificationDenial {
    Registration,
    Admission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationFrameCertificationDenial {
    Preparation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiDeclaredSurfaceCertificationDenial {
    UnknownDeclaration,
    Admission,
}

/// Certification-only observation of raw runtime plans and host capability authority.
pub trait WorthUiActiveSessionCertificationExt {
    fn inspect_runtime(&self) -> WorthUiActiveRuntimeObservation;

    fn inspect_query_state_residue(&self) -> WorthUiStateQueryResidueScan;

    fn refresh_query_change(
        &mut self,
        request: worth_ui_query_binding::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome,
        worth_ui_query_binding::WorthUiOperationLiveRefreshError,
    >;

    fn query_change_state(
        &self,
        reference: &worth_ui_query_binding::WorthUiInstalledQueryBindingReference,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveChangeObservation,
        worth_ui_query_binding::WorthUiQueryViewExecutionEvidenceDenial,
    >;

    fn measurement_basis_sources(
        &self,
    ) -> Box<[crate::declaration::UiDeclaredMeasurementBasisSource]>;

    fn ordinary_plan_availability(&self) -> WorthUiOrdinaryPlanAvailability;

    fn virtualized_plan_availability(&self) -> WorthUiVirtualizedPlanAvailability;

    fn query_fact_link(&self, binding_id: &str) -> Option<WorthUiQueryLaneFactLink>;

    fn canvas_spatial_plan_availability(&self) -> WorthUiCanvasSpatialPlanAvailability;

    fn first_canvas_spatial_handle(&self) -> Option<WorthUiLaneHandle>;

    fn inspect_canvas_spatial_target(
        &self,
        handle: WorthUiLaneHandle,
    ) -> Result<WorthUiCanvasSpatialTargetSummary, WorthUiCanvasSpatialInspectionDenial>;

    fn realtime_plan_availability(&self) -> WorthUiRealtimePlanAvailability;

    fn first_realtime_renderer_surface(&self) -> Option<WorthUiRendererSurfaceHandle>;

    fn inspect_realtime_target(
        &self,
        handle: WorthUiRendererSurfaceHandle,
    ) -> Result<WorthUiRealtimeTargetSummary, WorthUiRealtimeInspectionDenial>;

    fn inspect_virtualized_plan(
        &self,
        request: WorthUiVirtualizedPlanSummaryRequest,
    ) -> Result<WorthUiVirtualizedPlanSummary, WorthUiVirtualizedPlanSummaryDenial>;

    fn inspect_ordinary_plan(
        &self,
        request: WorthUiOrdinaryPlanSummaryRequest,
    ) -> Result<WorthUiOrdinaryPlanSummary, WorthUiOrdinaryPlanSummaryDenial>;

    fn host_measurement_capability(&self) -> WorthUiHostMeasurementCapability;

    fn create_declared_semantic_surface(
        &mut self,
        authored_name: &str,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        WorthUiDeclaredSurfaceCertificationDenial,
    >;

    fn declared_region_layout_inputs(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Box<[crate::facade::entry::UiNativeMountedRegionLayoutInput]>;

    fn lookup_consumed_fact(
        &self,
        fact: &UiProducedFact,
    ) -> Result<UiGraphFactLookupReceipt, UiGraphFactLookupDenial>;

    fn register_and_apply_component_semantic_text(
        &mut self,
        changes: &[crate::facade::entry::UiNativeComponentSemanticTextChange],
    ) -> Result<(), WorthUiSemanticTextCertificationDenial>;

    fn prepare_application_presentation_frame(
        &mut self,
        request: crate::facade::mounted::UiMountedFrameRequest,
    ) -> Result<
        crate::facade::mounted::UiPreparedMountedFrame,
        WorthUiPresentationFrameCertificationDenial,
    >;
}

impl WorthUiActiveSessionCertificationExt for WorthUiActiveApplicationSession {
    fn inspect_runtime(&self) -> WorthUiActiveRuntimeObservation {
        WorthUiActiveApplicationSession::inspect_runtime(self)
    }

    fn inspect_query_state_residue(&self) -> WorthUiStateQueryResidueScan {
        WorthUiActiveApplicationSession::inspect_query_state_residue(self)
    }

    fn refresh_query_change(
        &mut self,
        request: worth_ui_query_binding::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome,
        worth_ui_query_binding::WorthUiOperationLiveRefreshError,
    > {
        WorthUiActiveApplicationSession::refresh_query_change_for_certification(self, request)
    }

    fn query_change_state(
        &self,
        reference: &worth_ui_query_binding::WorthUiInstalledQueryBindingReference,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveChangeObservation,
        worth_ui_query_binding::WorthUiQueryViewExecutionEvidenceDenial,
    > {
        WorthUiActiveApplicationSession::query_change_state_for_certification(self, reference)
    }

    fn measurement_basis_sources(
        &self,
    ) -> Box<[crate::declaration::UiDeclaredMeasurementBasisSource]> {
        WorthUiActiveApplicationSession::measurement_basis_sources_for_certification(self)
    }

    fn ordinary_plan_availability(&self) -> WorthUiOrdinaryPlanAvailability {
        WorthUiActiveApplicationSession::ordinary_plan_availability(self)
    }

    fn virtualized_plan_availability(&self) -> WorthUiVirtualizedPlanAvailability {
        WorthUiActiveApplicationSession::virtualized_plan_availability(self)
    }

    fn query_fact_link(&self, binding_id: &str) -> Option<WorthUiQueryLaneFactLink> {
        WorthUiActiveApplicationSession::query_fact_link(self, binding_id)
    }

    fn canvas_spatial_plan_availability(&self) -> WorthUiCanvasSpatialPlanAvailability {
        WorthUiActiveApplicationSession::canvas_spatial_plan_availability(self)
    }

    fn first_canvas_spatial_handle(&self) -> Option<WorthUiLaneHandle> {
        WorthUiActiveApplicationSession::first_canvas_spatial_handle(self)
    }

    fn inspect_canvas_spatial_target(
        &self,
        handle: WorthUiLaneHandle,
    ) -> Result<WorthUiCanvasSpatialTargetSummary, WorthUiCanvasSpatialInspectionDenial> {
        WorthUiActiveApplicationSession::inspect_canvas_spatial_target(self, handle)
    }

    fn realtime_plan_availability(&self) -> WorthUiRealtimePlanAvailability {
        WorthUiActiveApplicationSession::realtime_plan_availability(self)
    }

    fn first_realtime_renderer_surface(&self) -> Option<WorthUiRendererSurfaceHandle> {
        WorthUiActiveApplicationSession::first_realtime_renderer_surface(self)
    }

    fn inspect_realtime_target(
        &self,
        handle: WorthUiRendererSurfaceHandle,
    ) -> Result<WorthUiRealtimeTargetSummary, WorthUiRealtimeInspectionDenial> {
        WorthUiActiveApplicationSession::inspect_realtime_target(self, handle)
    }

    fn inspect_virtualized_plan(
        &self,
        request: WorthUiVirtualizedPlanSummaryRequest,
    ) -> Result<WorthUiVirtualizedPlanSummary, WorthUiVirtualizedPlanSummaryDenial> {
        WorthUiActiveApplicationSession::inspect_virtualized_plan(self, request)
    }

    fn inspect_ordinary_plan(
        &self,
        request: WorthUiOrdinaryPlanSummaryRequest,
    ) -> Result<WorthUiOrdinaryPlanSummary, WorthUiOrdinaryPlanSummaryDenial> {
        WorthUiActiveApplicationSession::inspect_ordinary_plan(self, request)
    }

    fn host_measurement_capability(&self) -> WorthUiHostMeasurementCapability {
        WorthUiActiveApplicationSession::host_measurement_capability(self)
    }

    fn create_declared_semantic_surface(
        &mut self,
        authored_name: &str,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        WorthUiDeclaredSurfaceCertificationDenial,
    > {
        WorthUiActiveApplicationSession::create_declared_semantic_surface_named(
            self,
            authored_name,
        )
        .map_err(|denial| match denial {
            crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound => {
                WorthUiDeclaredSurfaceCertificationDenial::UnknownDeclaration
            }
            _ => WorthUiDeclaredSurfaceCertificationDenial::Admission,
        })
    }

    fn declared_region_layout_inputs(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Box<[crate::facade::entry::UiNativeMountedRegionLayoutInput]> {
        WorthUiActiveApplicationSession::declared_region_layout_inputs(self, surface)
    }

    fn lookup_consumed_fact(
        &self,
        fact: &UiProducedFact,
    ) -> Result<UiGraphFactLookupReceipt, UiGraphFactLookupDenial> {
        WorthUiActiveApplicationSession::lookup_consumed_fact_for_certification(self, fact)
    }

    fn register_and_apply_component_semantic_text(
        &mut self,
        changes: &[crate::facade::entry::UiNativeComponentSemanticTextChange],
    ) -> Result<(), WorthUiSemanticTextCertificationDenial> {
        let graph_nodes = {
            let graph = self.graph();
            graph
                .node_identities()
                .filter_map(|identity| {
                    let lookup = graph.lookup().graph_node(identity)?;
                    let semantic = lookup
                        .value()
                        .declaration_identity()
                        .authored_semantic_name()
                        .to_owned();
                    semantic
                        .starts_with("component:")
                        .then(|| (identity, Box::<str>::from(semantic)))
                })
                .collect::<Vec<_>>()
        };
        for (graph_node, authored_semantic_identity) in graph_nodes {
            WorthUiActiveApplicationSession::register_application_semantic_text(
                self,
                authored_semantic_identity,
                graph_node,
            )
            .map_err(|_| WorthUiSemanticTextCertificationDenial::Registration)?;
        }
        WorthUiActiveApplicationSession::admit_application_semantic_text(self, changes)
            .map_err(|_| WorthUiSemanticTextCertificationDenial::Admission)
    }

    fn prepare_application_presentation_frame(
        &mut self,
        request: crate::facade::mounted::UiMountedFrameRequest,
    ) -> Result<
        crate::facade::mounted::UiPreparedMountedFrame,
        WorthUiPresentationFrameCertificationDenial,
    > {
        self.prepare_mounted_frame_with_application_presentation(request, |_| {})
            .map_err(|_| WorthUiPresentationFrameCertificationDenial::Preparation)
    }
}
