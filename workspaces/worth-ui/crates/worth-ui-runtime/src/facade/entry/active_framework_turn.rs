use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use crate::runtime::WorthUiFrameworkTurnCompletion;

mod appearance_projection;
mod frame_completion;
mod mounted_projection;

pub use frame_completion::{
    WorthUiActiveCanvasSpatialFrameCompletion, WorthUiActiveOrdinaryFrameCompletion,
    WorthUiActiveRealtimeFrameCompletion, WorthUiActiveVirtualizedDataFrameCompletion,
};
pub use mounted_projection::WorthUiMountedLaneProjectionDenial;

/// One framework-turn result bound to the active application generation.
pub struct WorthUiActiveFrameworkTurnCompletion<'session> {
    pub(super) application_session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    pub(super) visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    pub(super) graph: crate::graph::UiGraphAuthority<'session>,
    pub(super) font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    pub(super) active_plan_digest: u64,
    pub(super) host_session_identity: crate::facade::WorthUiHostSessionIdentity,
    pub(super) completion: WorthUiFrameworkTurnCompletion<'session>,
    pub(super) capabilities: &'session crate::capability::CapabilitySnapshot,
    pub(super) mounted: &'session mut crate::mounting::WorthUiMountedSessionState,
    pub(super) host_session: &'session crate::facade::WorthUiHostSessionAuthority,
    pub(super) host_exchange: &'session mut crate::host_exchange::WorthUiHostExchangeSessionState,
    pub(super) focus: Option<&'session mut crate::runtime::focus::UiFocusRuntimeState>,
    pub(super) portal: Option<&'session mut crate::runtime::portal::UiPortalRuntimeState>,
    pub(super) interaction: &'session mut crate::runtime::interaction::UiInteractionRuntimeState,
    pub(super) presentation:
        &'session mut crate::runtime::presentation_state::UiApplicationPresentationState,
    pub(super) appearance_owner_snapshot:
        &'session Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pub(super) appearance_projection_transitions:
        &'session mut crate::runtime::appearance::UiAppearanceProjectionTransitionState,
    pub(super) appearance_inspection:
        &'session mut crate::runtime::appearance::UiAppearanceInspectionProducer,
}

/// Executable framework-turn authority lent by one active application session.
pub struct WorthUiActiveFrameworkTurnExecution<'session> {
    pub(super) application_session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    pub(super) visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    pub(super) graph: crate::graph::UiGraphAuthority<'session>,
    pub(super) font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    pub(super) host_session_identity: crate::facade::WorthUiHostSessionIdentity,
    pub(super) execution: crate::runtime::WorthUiFrameworkTurnExecution<'session>,
    pub(super) capabilities: &'session crate::capability::CapabilitySnapshot,
    pub(super) mounted: &'session mut crate::mounting::WorthUiMountedSessionState,
    pub(super) host_session: &'session crate::facade::WorthUiHostSessionAuthority,
    pub(super) host_exchange: &'session mut crate::host_exchange::WorthUiHostExchangeSessionState,
    pub(super) focus: Option<&'session mut crate::runtime::focus::UiFocusRuntimeState>,
    pub(super) portal: Option<&'session mut crate::runtime::portal::UiPortalRuntimeState>,
    pub(super) interaction: &'session mut crate::runtime::interaction::UiInteractionRuntimeState,
    pub(super) presentation:
        &'session mut crate::runtime::presentation_state::UiApplicationPresentationState,
    pub(super) appearance_owner_snapshot:
        &'session Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pub(super) appearance_projection_transitions:
        &'session mut crate::runtime::appearance::UiAppearanceProjectionTransitionState,
    pub(super) appearance_inspection:
        &'session mut crate::runtime::appearance::UiAppearanceInspectionProducer,
    pub(super) host_protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    pub(super) host_capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(super) host_capability_profile_digest: u64,
}

impl<'session> WorthUiActiveFrameworkTurnCompletion<'session> {
    pub fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.generation_identity
    }

    pub fn into_completion(self) -> WorthUiFrameworkTurnCompletion<'session> {
        self.completion
    }

    pub fn into_execution(
        self,
    ) -> Result<WorthUiActiveFrameworkTurnExecution<'session>, Box<Self>> {
        let Self {
            application_session_identity,
            generation_identity,
            visual_trace_source,
            graph,
            font_collection,
            active_plan_digest,
            host_session_identity,
            completion,
            capabilities,
            mounted,
            host_session,
            host_exchange,
            focus,
            portal,
            interaction,
            presentation,
            appearance_owner_snapshot,
            appearance_projection_transitions,
            appearance_inspection,
        } = self;
        let host_protocol = host_session.protocol();
        let capability_report = host_session.capability_report();
        match completion.into_execution() {
            Ok(execution) => Ok(WorthUiActiveFrameworkTurnExecution {
                application_session_identity,
                generation_identity,
                visual_trace_source,
                graph,
                font_collection: std::sync::Arc::clone(&font_collection),
                host_session_identity,
                execution,
                capabilities,
                mounted,
                host_session,
                host_exchange,
                focus,
                portal,
                interaction,
                presentation,
                appearance_owner_snapshot,
                appearance_projection_transitions,
                appearance_inspection,
                host_protocol,
                host_capability_generation: capability_report.observation_generation(),
                host_capability_profile_digest: capability_report.profile_identity_digest(),
            }),
            Err(completion) => Err(Box::new(Self {
                application_session_identity,
                generation_identity,
                visual_trace_source,
                graph,
                font_collection,
                active_plan_digest,
                host_session_identity,
                completion: *completion,
                capabilities,
                mounted,
                host_session,
                host_exchange,
                focus,
                portal,
                interaction,
                presentation,
                appearance_owner_snapshot,
                appearance_projection_transitions,
                appearance_inspection,
            })),
        }
    }
}

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub fn into_activation_boundary(self) -> crate::runtime::WorthUiFrameBoundary {
        self.execution.into_activation_boundary()
    }

    pub fn execute_ordinary_frame(
        &self,
        target: crate::runtime::WorthUiOrdinaryFrameTarget,
    ) -> Result<
        WorthUiActiveOrdinaryFrameCompletion<'_>,
        crate::runtime::WorthUiOrdinaryLaneFrameDenial,
    > {
        let receipt = self.execution.execute_active_ordinary_frame(target)?;
        Ok(WorthUiActiveOrdinaryFrameCompletion::new(
            &self.generation_identity,
            receipt,
            self.frame_execution_basis(),
        ))
    }

    pub fn execute_canvas_spatial_frame(
        &self,
        target: crate::runtime::WorthUiCanvasSpatialFrameTarget,
    ) -> Result<
        WorthUiActiveCanvasSpatialFrameCompletion<'_>,
        crate::runtime::WorthUiCanvasSpatialFrameDenial,
    > {
        let receipt = self.execution.execute_active_canvas_spatial_frame(target)?;
        Ok(WorthUiActiveCanvasSpatialFrameCompletion::new(
            &self.generation_identity,
            receipt,
            self.frame_execution_basis(),
        ))
    }

    pub fn execute_realtime_frame(
        &self,
        target: crate::runtime::WorthUiRealtimeFrameTarget,
    ) -> Result<WorthUiActiveRealtimeFrameCompletion<'_>, crate::runtime::WorthUiRealtimeFrameDenial>
    {
        let receipt = self.execution.execute_active_realtime_frame(target)?;
        Ok(WorthUiActiveRealtimeFrameCompletion::new(
            &self.generation_identity,
            receipt,
            self.frame_execution_basis(),
        ))
    }

    pub fn execute_virtualized_data_frame(
        &self,
        target: crate::runtime::WorthUiVirtualizedDataFrameTarget,
    ) -> Result<
        WorthUiActiveVirtualizedDataFrameCompletion<'_>,
        crate::runtime::WorthUiVirtualizedDataFrameDenial,
    > {
        let receipt = self
            .execution
            .execute_active_virtualized_data_frame(target)?;
        Ok(WorthUiActiveVirtualizedDataFrameCompletion::new(
            &self.generation_identity,
            receipt,
            self.frame_execution_basis(),
        ))
    }

    fn frame_execution_basis(&self) -> crate::runtime::WorthUiFrameExecutionBasis {
        crate::runtime::WorthUiFrameExecutionBasis::new(
            self.host_session_identity.as_u64(),
            self.execution.active_artifact_digest(),
            self.execution.active_plan_digest(),
            self.execution.active_frame_epoch().as_u64(),
        )
    }
}
