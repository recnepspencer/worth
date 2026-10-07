mod admission_evidence;
mod counters;
mod digests;
mod input_kind;
mod lowering;
mod parity;
mod plan;
mod preflight;
mod reports;
mod routes;

pub(crate) use digests::BundleResolvedBasisDigest;
pub use digests::FrontierBreadthPrediction;
pub use digests::FrontierComplexityContract;
pub use digests::FrontierPerformanceStatus;
pub use digests::FrontierPostureDigest;
pub use digests::FrontierPredictionDriftOutcome;
pub use digests::FrontierSurfaceDigest;
pub(crate) use digests::PacketEquivalenceContract;
pub(crate) use digests::PacketMergeBoundary;
pub(crate) use digests::PacketMergeContract;
pub(crate) use digests::PlannedWorkPacketDigest;

pub use counters::FrontierCounterSnapshot;
pub(crate) use counters::FrontierPlanFamily;
pub use counters::FrontierPlanningCounters;
pub use counters::FrontierRouteCounters;
pub(crate) use counters::PlannedWorkPacket;
pub(crate) use counters::PlannedWorkPacketFamily;
pub(crate) use counters::PlannedWorkPacketSet;

pub use parity::FrontierParityBundle;
pub use parity::FrontierParityBundleError;
pub use parity::PlannedRouteFamily;

pub use reports::FrontierPlanningReport;
pub use reports::FrontierRouteReport;

pub use plan::FrontierAwarePlan;
pub(crate) use plan::FrontierBundlePlan;
pub(crate) use plan::FrontierPlanningError;
pub(crate) use plan::FrontierPlanningInput;

pub use admission_evidence::SerialFallbackBundleEvidence;
pub(crate) use admission_evidence::SerialFallbackBundleEvidenceError;
pub use admission_evidence::SerialFallbackEvidence;
pub use admission_evidence::SerialFallbackReason;

pub use preflight::BoundedMaterializationFrontierPreflight;
pub use preflight::FrontierPreflightAdmissionError;

pub use routes::FrontierBundleRoutePlanningError;
pub use routes::FrontierRoutePlanningError;
pub use routes::SerialFallbackBundleRoutes;
pub use routes::SerialFallbackRoute;

pub use lowering::admit_bounded_materialization_frontier_preflight;
pub(crate) use lowering::lower_frontier_bundle;
pub(crate) use lowering::lower_live_plan_to_frontier_plan;
pub use lowering::lower_preflight_bundle_to_serial_fallback_routes;
pub(crate) use lowering::lower_preflight_to_frontier_plan;
pub use lowering::lower_preflight_to_serial_fallback_route;

pub(in crate::frontier_planning::testing) use input_kind::frontier_input_kind;
