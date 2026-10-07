use crate::basis::ExecutionPreflightBundle;
use crate::identity::{PlanDigest, ValidatedQueryDigest};
use crate::live::LiveQueryPlan;

use super::{
    BundleResolvedBasisDigest, FrontierBreadthPrediction, FrontierComplexityContract,
    FrontierPerformanceStatus, FrontierPlanFamily, FrontierPlanningCounters,
    FrontierPlanningReport, FrontierPredictionDriftOutcome, PlannedWorkPacketSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierAwarePlan {
    pub(in crate::frontier_planning::testing) query_digest: ValidatedQueryDigest,
    pub(in crate::frontier_planning::testing) source_plan_digest: PlanDigest,
    pub(in crate::frontier_planning::testing) family: FrontierPlanFamily,
    pub(in crate::frontier_planning::testing) bundle_basis_digest: BundleResolvedBasisDigest,
    pub(in crate::frontier_planning::testing) packet_set: PlannedWorkPacketSet,
    pub(in crate::frontier_planning::testing) predicted_breadth: FrontierBreadthPrediction,
    pub(in crate::frontier_planning::testing) drift_outcome: FrontierPredictionDriftOutcome,
    pub(in crate::frontier_planning::testing) complexity_contract: FrontierComplexityContract,
    pub(in crate::frontier_planning::testing) performance_status: FrontierPerformanceStatus,
    pub(in crate::frontier_planning::testing) report: FrontierPlanningReport,
    pub(in crate::frontier_planning::testing) counters: FrontierPlanningCounters,
}

impl FrontierAwarePlan {
    pub fn query_digest(&self) -> &ValidatedQueryDigest {
        &self.query_digest
    }

    pub fn source_plan_digest(&self) -> &PlanDigest {
        &self.source_plan_digest
    }

    pub(crate) fn family(&self) -> &FrontierPlanFamily {
        &self.family
    }

    pub(crate) fn bundle_basis_digest(&self) -> &BundleResolvedBasisDigest {
        &self.bundle_basis_digest
    }

    pub(crate) fn packet_set(&self) -> &PlannedWorkPacketSet {
        &self.packet_set
    }

    pub fn predicted_breadth(&self) -> &FrontierBreadthPrediction {
        &self.predicted_breadth
    }

    pub fn drift_outcome(&self) -> &FrontierPredictionDriftOutcome {
        &self.drift_outcome
    }

    pub fn complexity_contract(&self) -> &FrontierComplexityContract {
        &self.complexity_contract
    }

    pub fn performance_status(&self) -> &FrontierPerformanceStatus {
        &self.performance_status
    }

    pub fn report(&self) -> &FrontierPlanningReport {
        &self.report
    }

    pub fn counters(&self) -> &FrontierPlanningCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrontierBundlePlan {
    pub(in crate::frontier_planning::testing) bundle_basis_digest: BundleResolvedBasisDigest,
    pub(in crate::frontier_planning::testing) route_plans: Vec<FrontierAwarePlan>,
    pub(in crate::frontier_planning::testing) counters: FrontierPlanningCounters,
}

impl FrontierBundlePlan {
    pub(crate) fn bundle_basis_digest(&self) -> &BundleResolvedBasisDigest {
        &self.bundle_basis_digest
    }

    pub(crate) fn route_plans(&self) -> &[FrontierAwarePlan] {
        &self.route_plans
    }

    pub(crate) fn counters(&self) -> &FrontierPlanningCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrontierPlanningError {
    UnsupportedFrontierFamily,
    UnsupportedBundleComposition,
    MixedBasisBundle {
        expected_basis_digest: BundleResolvedBasisDigest,
        found_basis_digest: BundleResolvedBasisDigest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrontierPlanningInput {
    ExecutionPreflight(ExecutionPreflightBundle),
    LivePlan(LiveQueryPlan),
}

impl From<ExecutionPreflightBundle> for FrontierPlanningInput {
    fn from(value: ExecutionPreflightBundle) -> Self {
        Self::ExecutionPreflight(value)
    }
}

impl From<LiveQueryPlan> for FrontierPlanningInput {
    fn from(value: LiveQueryPlan) -> Self {
        Self::LivePlan(value)
    }
}
