#[cfg(test)]
pub(crate) use crate::frontier_planning::{
    BoundedMaterializationFrontierPreflight, FrontierAwarePlan, FrontierBundlePlan,
    FrontierBundleRoutePlanningError, FrontierCounterSnapshot, FrontierParityBundle,
    FrontierPlanFamily, FrontierPlanningError, FrontierPlanningInput,
    FrontierPredictionDriftOutcome, FrontierPreflightAdmissionError, FrontierRoutePlanningError,
    FrontierSurfaceDigest, PacketMergeContract, PlannedRouteFamily, PlannedWorkPacketFamily,
    SerialFallbackBundleEvidence, SerialFallbackBundleRoutes, SerialFallbackEvidence,
    SerialFallbackReason, SerialFallbackRoute,
};

#[cfg(test)]
pub(crate) fn lower_execution_preflight_to_frontier_plan(
    preflight: &crate::basis::ExecutionPreflightBundle,
) -> Result<FrontierAwarePlan, FrontierPlanningError> {
    crate::frontier_planning::lower_preflight_to_frontier_plan(preflight)
}

#[cfg(test)]
pub(crate) fn lower_live_plan_to_frontier_plan(
    live: &crate::live::LiveQueryPlan,
) -> Result<FrontierAwarePlan, FrontierPlanningError> {
    crate::frontier_planning::lower_live_plan_to_frontier_plan(live)
}

#[cfg(test)]
pub(crate) fn lower_frontier_planning_bundle(
    inputs: &[FrontierPlanningInput],
) -> Result<FrontierBundlePlan, FrontierPlanningError> {
    crate::frontier_planning::lower_frontier_bundle(inputs)
}

#[cfg(test)]
pub(crate) fn lower_preflight_to_serial_fallback_route(
    preflight: &BoundedMaterializationFrontierPreflight,
    evidence: &SerialFallbackEvidence,
) -> Result<SerialFallbackRoute, FrontierRoutePlanningError> {
    crate::frontier_planning::lower_preflight_to_serial_fallback_route(preflight, evidence)
}

#[cfg(test)]
pub(crate) fn lower_preflight_bundle_to_serial_fallback_routes(
    preflights: &[BoundedMaterializationFrontierPreflight],
    evidences: &crate::frontier_planning::SerialFallbackBundleEvidence,
) -> Result<SerialFallbackBundleRoutes, FrontierBundleRoutePlanningError> {
    crate::frontier_planning::lower_preflight_bundle_to_serial_fallback_routes(
        preflights, evidences,
    )
}

#[cfg(test)]
pub(crate) fn admit_bounded_materialization_frontier_preflight(
    preflight: crate::basis::ExecutionPreflightBundle,
) -> Result<BoundedMaterializationFrontierPreflight, FrontierPreflightAdmissionError> {
    crate::frontier_planning::admit_bounded_materialization_frontier_preflight(preflight)
}
