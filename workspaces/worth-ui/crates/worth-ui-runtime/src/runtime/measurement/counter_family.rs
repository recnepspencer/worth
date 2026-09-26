use super::measurement_boundary::WorthUiMeasurementBoundary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorthUiCounterAuthority {
    WorthUiRuntime,
    WorthQueryEvidence,
    DiagnosticsProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorthUiRuntimeCounterFamily {
    ReloadCandidateAdmission,
    SourceIngress,
    ArtifactComparison,
    ImpactNarrowing,
    IdentityReplacement,
    DurableStateReconciliation,
    QueryRebindPlanning,
    PlanAssembly,
    PlanLowering,
    LaneAdmission,
    OrdinaryLaneExecution,
    VirtualizedDataExecution,
    CanvasSpatialExecution,
    RealtimeOverlayExecution,
    Activation,
    CommittedAllocationActivation,
    SteadyFrameRendering,
    DiagnosticsProjection,
}

impl WorthUiRuntimeCounterFamily {
    pub fn reload_candidate_admission() -> Self {
        Self::ReloadCandidateAdmission
    }

    pub fn source_ingress() -> Self {
        Self::SourceIngress
    }

    pub fn plan_lowering() -> Self {
        Self::PlanLowering
    }

    pub fn ordinary_lane_execution() -> Self {
        Self::OrdinaryLaneExecution
    }

    pub fn steady_frame_rendering() -> Self {
        Self::SteadyFrameRendering
    }

    pub fn diagnostics_projection() -> Self {
        Self::DiagnosticsProjection
    }

    pub fn at_boundary(
        self,
        boundary: WorthUiMeasurementBoundary,
    ) -> super::counter_packet::WorthUiCounterPacketBuilder {
        super::counter_packet::WorthUiCounterPacketBuilder::new(self, boundary)
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::ReloadCandidateAdmission => "reload.candidate_admission",
            Self::SourceIngress => "reload.source_ingress",
            Self::ArtifactComparison => "reload.artifact_comparison",
            Self::ImpactNarrowing => "reload.impact_narrowing",
            Self::IdentityReplacement => "reload.identity_replacement",
            Self::DurableStateReconciliation => "reload.durable_state_reconciliation",
            Self::QueryRebindPlanning => "reload.query_rebind_planning",
            Self::PlanAssembly => "plan.assembly",
            Self::PlanLowering => "plan.lowering",
            Self::LaneAdmission => "lane.admission",
            Self::OrdinaryLaneExecution => "lane.ordinary.execution",
            Self::VirtualizedDataExecution => "lane.virtualized_data.execution",
            Self::CanvasSpatialExecution => "lane.canvas_spatial.execution",
            Self::RealtimeOverlayExecution => "lane.realtime_overlay.execution",
            Self::Activation => "activation",
            Self::CommittedAllocationActivation => "activation.committed_allocation",
            Self::SteadyFrameRendering => "frame.steady_rendering",
            Self::DiagnosticsProjection => "diagnostics.projection",
        }
    }

    pub fn authority(self) -> WorthUiCounterAuthority {
        match self {
            Self::QueryRebindPlanning => WorthUiCounterAuthority::WorthUiRuntime,
            Self::DiagnosticsProjection => WorthUiCounterAuthority::DiagnosticsProjection,
            Self::ReloadCandidateAdmission
            | Self::SourceIngress
            | Self::ArtifactComparison
            | Self::ImpactNarrowing
            | Self::IdentityReplacement
            | Self::DurableStateReconciliation
            | Self::PlanAssembly
            | Self::PlanLowering
            | Self::LaneAdmission
            | Self::OrdinaryLaneExecution
            | Self::VirtualizedDataExecution
            | Self::CanvasSpatialExecution
            | Self::RealtimeOverlayExecution
            | Self::Activation
            | Self::CommittedAllocationActivation
            | Self::SteadyFrameRendering => WorthUiCounterAuthority::WorthUiRuntime,
        }
    }

    pub fn allowed_boundary(self) -> WorthUiMeasurementBoundary {
        match self {
            Self::ReloadCandidateAdmission => WorthUiMeasurementBoundary::ReloadCandidateAdmission,
            Self::SourceIngress => WorthUiMeasurementBoundary::SourceIngress,
            Self::ArtifactComparison => WorthUiMeasurementBoundary::ArtifactComparison,
            Self::ImpactNarrowing => WorthUiMeasurementBoundary::ImpactNarrowing,
            Self::IdentityReplacement => WorthUiMeasurementBoundary::IdentityReplacement,
            Self::DurableStateReconciliation => {
                WorthUiMeasurementBoundary::DurableStateReconciliation
            }
            Self::QueryRebindPlanning => WorthUiMeasurementBoundary::QueryRebindPlanning,
            Self::PlanAssembly => WorthUiMeasurementBoundary::PlanAssembly,
            Self::PlanLowering => WorthUiMeasurementBoundary::PlanLowering,
            Self::LaneAdmission => WorthUiMeasurementBoundary::LaneAdmission,
            Self::OrdinaryLaneExecution => WorthUiMeasurementBoundary::OrdinaryLaneExecution,
            Self::VirtualizedDataExecution => WorthUiMeasurementBoundary::VirtualizedDataExecution,
            Self::CanvasSpatialExecution => WorthUiMeasurementBoundary::CanvasSpatialExecution,
            Self::RealtimeOverlayExecution => WorthUiMeasurementBoundary::RealtimeOverlayExecution,
            Self::Activation => WorthUiMeasurementBoundary::Activation,
            Self::CommittedAllocationActivation => {
                WorthUiMeasurementBoundary::CommittedAllocationActivation
            }
            Self::SteadyFrameRendering => WorthUiMeasurementBoundary::SteadyFrameRendering,
            Self::DiagnosticsProjection => WorthUiMeasurementBoundary::DiagnosticsProjection,
        }
    }
}
