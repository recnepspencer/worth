mod artifacts;
mod counters;
mod errors;
mod execution_bundle;
#[cfg(test)]
mod frontier_lowering;
mod report;
mod request_context;
mod route;
mod seed;
mod selection;

pub use artifacts::{PlannedQueryArtifact, PlannedResultShapeArtifact};
pub use counters::PlanningCounters;
pub use errors::{PlanningError, PlanningFailureClass};
pub use execution_bundle::ExecutionPlanBundle;
pub use report::PlanningReport;
pub use request_context::{
    planning_request_context_for_bound, planning_request_context_for_direct,
    PlanningAmbientContext, PlanningRequestContext, PlanningSemanticInputs,
};
pub use route::{
    ExecutionCostMarker, ExecutionMechanics, FallbackDisposition, PlannedExecutionRoute,
};
pub use seed::seed_execution_plan;

pub(crate) use selection::{
    plan_validated_bundle, plan_validated_bundle_for_collection_family,
    plan_validated_bundle_for_count_aggregate,
    plan_validated_bundle_for_count_aggregate_with_policy_authority,
    plan_validated_bundle_with_policy_authority,
};

#[cfg(test)]
pub(crate) use frontier_lowering::{
    FrontierBundleRoutePlanningError, FrontierCounterSnapshot, FrontierParityBundle,
    FrontierPlanFamily, FrontierPlanningError, FrontierPlanningInput,
    FrontierPredictionDriftOutcome, FrontierRoutePlanningError, FrontierSurfaceDigest,
    PacketMergeContract, PlannedRouteFamily, PlannedWorkPacketFamily, SerialFallbackBundleEvidence,
    SerialFallbackEvidence, SerialFallbackReason, SerialFallbackRoute,
};

#[cfg(test)]
pub(crate) use frontier_lowering::{
    admit_bounded_materialization_frontier_preflight, lower_execution_preflight_to_frontier_plan,
    lower_frontier_planning_bundle, lower_live_plan_to_frontier_plan,
    lower_preflight_bundle_to_serial_fallback_routes, lower_preflight_to_serial_fallback_route,
};
