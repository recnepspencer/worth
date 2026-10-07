use crate::basis::ExecutionPreflightBundle;
use crate::identity::{PlanDigest, ValidatedQueryDigest};

use super::{
    BundleResolvedBasisDigest, FrontierAwarePlan, FrontierPlanningCounters, FrontierPlanningError,
    FrontierPostureDigest, FrontierRouteCounters, FrontierRouteReport,
    SerialFallbackBundleEvidence, SerialFallbackEvidence, SerialFallbackReason,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialFallbackRoute {
    preflight: ExecutionPreflightBundle,
    frontier_plan: FrontierAwarePlan,
    report: FrontierRouteReport,
    planning_counters: FrontierPlanningCounters,
    counters: FrontierRouteCounters,
    route_posture_digest: FrontierPostureDigest,
}

impl SerialFallbackRoute {
    pub fn query_digest(&self) -> &ValidatedQueryDigest {
        self.frontier_plan.query_digest()
    }

    pub fn source_plan_digest(&self) -> &PlanDigest {
        self.frontier_plan.source_plan_digest()
    }

    pub fn posture_digest(&self) -> &FrontierPostureDigest {
        &self.route_posture_digest
    }

    pub fn reason(&self) -> &SerialFallbackReason {
        self.report.serial_fallback_reason()
    }

    pub fn report(&self) -> &FrontierRouteReport {
        &self.report
    }

    pub(crate) fn planning_counters(&self) -> &FrontierPlanningCounters {
        &self.planning_counters
    }

    pub fn counters(&self) -> &FrontierRouteCounters {
        &self.counters
    }

    pub(crate) fn preflight(&self) -> &ExecutionPreflightBundle {
        &self.preflight
    }

    pub(in crate::frontier_planning::testing) fn new(
        preflight: ExecutionPreflightBundle,
        frontier_plan: FrontierAwarePlan,
        evidence: &SerialFallbackEvidence,
    ) -> Self {
        let route_posture_digest = evidence.route_posture_digest(&frontier_plan);
        let report = FrontierRouteReport::from_serial_route(
            route_posture_digest.clone(),
            &frontier_plan,
            evidence,
        );
        let counters = FrontierRouteCounters::serial(evidence.drift_outcome());
        Self {
            preflight,
            planning_counters: frontier_plan.counters().clone(),
            frontier_plan,
            report,
            counters,
            route_posture_digest,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialFallbackBundleRoutes {
    bundle_basis_digest: BundleResolvedBasisDigest,
    bundle_posture_digest: FrontierPostureDigest,
    planning_counters: FrontierPlanningCounters,
    routes: Vec<SerialFallbackRoute>,
}

impl SerialFallbackBundleRoutes {
    pub fn bundle_basis_digest(&self) -> &str {
        self.bundle_basis_digest.as_str()
    }

    pub fn bundle_posture_digest(&self) -> &FrontierPostureDigest {
        &self.bundle_posture_digest
    }

    pub fn routes(&self) -> &[SerialFallbackRoute] {
        &self.routes
    }

    pub(crate) fn planning_counters(&self) -> &FrontierPlanningCounters {
        &self.planning_counters
    }

    pub(in crate::frontier_planning::testing) fn new(
        bundle_basis_digest: BundleResolvedBasisDigest,
        planning_counters: FrontierPlanningCounters,
        bundle_evidence: &SerialFallbackBundleEvidence,
        routes: Vec<SerialFallbackRoute>,
    ) -> Self {
        let mut parts = vec![
            format!("bundle_basis:{}", bundle_basis_digest.as_str()),
            format!(
                "bundle_surface:{}",
                bundle_evidence.bundle_surface_digest().as_str()
            ),
        ];
        for (index, route) in routes.iter().enumerate() {
            parts.push(format!(
                "route[{index}]:{}",
                route.posture_digest().as_str()
            ));
        }
        Self {
            bundle_basis_digest,
            bundle_posture_digest: FrontierPostureDigest::from_parts(&parts),
            planning_counters,
            routes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontierRoutePlanningError {
    UnsupportedFrontierFamily,
    PredictionDriftDenied {
        posture_digest: FrontierPostureDigest,
    },
    SerialFallbackUnavailable {
        posture_digest: FrontierPostureDigest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontierBundleRoutePlanningError {
    UnsupportedBundleComposition,
    MixedBasisBundle {
        expected_basis_digest: String,
        found_basis_digest: String,
    },
    EvidenceCountMismatch {
        expected: usize,
        found: usize,
    },
    RoutePlanningFailed {
        route_index: usize,
        error: FrontierRoutePlanningError,
    },
}

impl From<FrontierPlanningError> for FrontierRoutePlanningError {
    fn from(value: FrontierPlanningError) -> Self {
        match value {
            FrontierPlanningError::UnsupportedFrontierFamily
            | FrontierPlanningError::UnsupportedBundleComposition
            | FrontierPlanningError::MixedBasisBundle { .. } => Self::UnsupportedFrontierFamily,
        }
    }
}
