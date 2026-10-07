use crate::basis::ExecutionPreflightBundle;
use crate::execution::ExecutionResultEnvelope;
use crate::identity::{PlanDigest, ResultDigest, ValidatedQueryDigest};

use super::{
    FrontierAwarePlan, FrontierBreadthPrediction, FrontierCounterSnapshot, FrontierPostureDigest,
    SerialFallbackBundleRoutes, SerialFallbackRoute,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PlannedRouteFamily {
    FrontierSerialControl,
    FrontierSerialFallback,
    FrontierSerialFallbackBundle,
}

impl PlannedRouteFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FrontierSerialControl => "frontier_serial_control",
            Self::FrontierSerialFallback => "frontier_serial_fallback",
            Self::FrontierSerialFallbackBundle => "frontier_serial_fallback_bundle",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontierParityBundleError {
    BundleRouteIndexOutOfRange {
        route_count: usize,
        route_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierParityBundle {
    query_digest: ValidatedQueryDigest,
    plan_digest: PlanDigest,
    result_digest: ResultDigest,
    basis_digest: String,
    route_family: PlannedRouteFamily,
    route_posture_digest: FrontierPostureDigest,
    predicted_breadth: FrontierBreadthPrediction,
    reported_execution_records_examined_count: usize,
    counter_snapshot: FrontierCounterSnapshot,
}

impl FrontierParityBundle {
    pub fn query_digest(&self) -> &ValidatedQueryDigest {
        &self.query_digest
    }

    pub fn plan_digest(&self) -> &PlanDigest {
        &self.plan_digest
    }

    pub fn result_digest(&self) -> &ResultDigest {
        &self.result_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn route_family(&self) -> &PlannedRouteFamily {
        &self.route_family
    }

    pub fn route_posture_digest(&self) -> &FrontierPostureDigest {
        &self.route_posture_digest
    }

    pub fn predicted_breadth(&self) -> &FrontierBreadthPrediction {
        &self.predicted_breadth
    }

    pub fn reported_execution_records_examined_count(&self) -> usize {
        self.reported_execution_records_examined_count
    }

    pub fn counter_snapshot(&self) -> &FrontierCounterSnapshot {
        &self.counter_snapshot
    }

    pub fn from_serial_control(
        frontier_plan: &FrontierAwarePlan,
        preflight: &ExecutionPreflightBundle,
        execution: &ExecutionResultEnvelope,
    ) -> Self {
        Self {
            query_digest: frontier_plan.query_digest().clone(),
            plan_digest: frontier_plan.source_plan_digest().clone(),
            result_digest: execution.report().result_digest().clone(),
            basis_digest: preflight.basis().proof().digest().as_str().to_string(),
            route_family: PlannedRouteFamily::FrontierSerialControl,
            route_posture_digest: frontier_plan.report().posture_digest().clone(),
            predicted_breadth: frontier_plan.predicted_breadth().clone(),
            reported_execution_records_examined_count: execution
                .counters()
                .execution_records_examined_count(),
            counter_snapshot: FrontierCounterSnapshot::serial_control(
                frontier_plan.counters(),
                execution.counters(),
            ),
        }
    }

    pub fn from_serial_fallback(
        route: &SerialFallbackRoute,
        execution: &ExecutionResultEnvelope,
    ) -> Self {
        Self {
            query_digest: route.query_digest().clone(),
            plan_digest: route.source_plan_digest().clone(),
            result_digest: execution.report().result_digest().clone(),
            basis_digest: route
                .preflight()
                .basis()
                .proof()
                .digest()
                .as_str()
                .to_string(),
            route_family: PlannedRouteFamily::FrontierSerialFallback,
            route_posture_digest: route.posture_digest().clone(),
            predicted_breadth: route.report().predicted_breadth().clone(),
            reported_execution_records_examined_count: execution
                .counters()
                .execution_records_examined_count(),
            counter_snapshot: FrontierCounterSnapshot::serial_fallback(
                route.planning_counters(),
                route.counters(),
                execution.counters(),
            ),
        }
    }

    pub fn from_serial_fallback_bundle(
        bundle: &SerialFallbackBundleRoutes,
        route_index: usize,
        execution: &ExecutionResultEnvelope,
    ) -> Result<Self, FrontierParityBundleError> {
        let route = bundle.routes().get(route_index).ok_or(
            FrontierParityBundleError::BundleRouteIndexOutOfRange {
                route_count: bundle.routes().len(),
                route_index,
            },
        )?;
        Ok(Self {
            query_digest: route.query_digest().clone(),
            plan_digest: route.source_plan_digest().clone(),
            result_digest: execution.report().result_digest().clone(),
            basis_digest: bundle.bundle_basis_digest().to_string(),
            route_family: PlannedRouteFamily::FrontierSerialFallbackBundle,
            route_posture_digest: bundle.bundle_posture_digest().clone(),
            predicted_breadth: route.report().predicted_breadth().clone(),
            reported_execution_records_examined_count: execution
                .counters()
                .execution_records_examined_count(),
            counter_snapshot: FrontierCounterSnapshot::serial_fallback_bundle(
                bundle.planning_counters(),
                route.counters(),
                execution.counters(),
                bundle.routes().len(),
            ),
        })
    }
}
