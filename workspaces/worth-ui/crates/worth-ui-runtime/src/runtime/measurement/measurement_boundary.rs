use worth_foundational::FoundationalPerformanceBoundary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorthUiMeasurementBoundary {
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

impl WorthUiMeasurementBoundary {
    pub fn reload_candidate_admission() -> Self {
        Self::ReloadCandidateAdmission
    }

    pub fn plan_lowering() -> Self {
        Self::PlanLowering
    }

    pub fn steady_frame_rendering() -> Self {
        Self::SteadyFrameRendering
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

    pub fn foundational_boundary(self) -> FoundationalPerformanceBoundary {
        match self {
            Self::DiagnosticsProjection => FoundationalPerformanceBoundary::BoundaryMaterialization,
            Self::PlanAssembly | Self::PlanLowering | Self::LaneAdmission => {
                FoundationalPerformanceBoundary::MaintenanceExecution
            }
            Self::ReloadCandidateAdmission
            | Self::SourceIngress
            | Self::ArtifactComparison
            | Self::ImpactNarrowing
            | Self::IdentityReplacement
            | Self::DurableStateReconciliation
            | Self::QueryRebindPlanning
            | Self::OrdinaryLaneExecution
            | Self::VirtualizedDataExecution
            | Self::CanvasSpatialExecution
            | Self::RealtimeOverlayExecution
            | Self::Activation
            | Self::CommittedAllocationActivation
            | Self::SteadyFrameRendering => FoundationalPerformanceBoundary::AuthoritativeExecution,
        }
    }
}
