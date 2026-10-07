use crate::execution::ExecutionCounters;

use super::super::{FrontierPlanningCounters, FrontierRouteCounters};

// Planned shape and executor-reported records; these are not execution or denial counts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierCounterSnapshot {
    planned_frontier_breadth: usize,
    reported_execution_records_examined_count: usize,
    planned_serial_fallback_route_count: usize,
    bundle_serial_route_count: usize,
    planned_packet_merge_boundary_count: usize,
    selected_route_nonbudget_drift_posture_count: usize,
}

impl FrontierCounterSnapshot {
    pub(crate) fn serial_control(
        planning: &FrontierPlanningCounters,
        execution: &ExecutionCounters,
    ) -> Self {
        Self {
            planned_frontier_breadth: planning.predicted_breadth(),
            reported_execution_records_examined_count: execution.execution_records_examined_count(),
            planned_serial_fallback_route_count: 0,
            bundle_serial_route_count: 0,
            planned_packet_merge_boundary_count: planning.planned_packet_merge_boundary_count(),
            selected_route_nonbudget_drift_posture_count: 0,
        }
    }

    pub(crate) fn serial_fallback(
        planning: &FrontierPlanningCounters,
        route: &FrontierRouteCounters,
        execution: &ExecutionCounters,
    ) -> Self {
        Self {
            planned_frontier_breadth: planning.predicted_breadth(),
            reported_execution_records_examined_count: execution.execution_records_examined_count(),
            planned_serial_fallback_route_count: route.route_serial_fallback_count(),
            bundle_serial_route_count: 0,
            planned_packet_merge_boundary_count: planning.planned_packet_merge_boundary_count(),
            selected_route_nonbudget_drift_posture_count: route
                .route_nonbudget_drift_posture_count(),
        }
    }

    pub(crate) fn serial_fallback_bundle(
        planning: &FrontierPlanningCounters,
        route: &FrontierRouteCounters,
        execution: &ExecutionCounters,
        bundle_serial_route_count: usize,
    ) -> Self {
        let mut snapshot = Self::serial_fallback(planning, route, execution);
        snapshot.bundle_serial_route_count = bundle_serial_route_count;
        snapshot.planned_serial_fallback_route_count = bundle_serial_route_count;
        snapshot
    }

    pub(crate) fn digest_parts(&self, label: &str) -> Vec<String> {
        vec![
            format!(
                "{label}.planned_frontier_breadth:{}",
                self.planned_frontier_breadth
            ),
            format!(
                "{label}.reported_execution_records_examined_count:{}",
                self.reported_execution_records_examined_count
            ),
            format!(
                "{label}.planned_serial_fallback_route_count:{}",
                self.planned_serial_fallback_route_count
            ),
            format!(
                "{label}.bundle_serial_route_count:{}",
                self.bundle_serial_route_count
            ),
            format!(
                "{label}.planned_packet_merge_boundary_count:{}",
                self.planned_packet_merge_boundary_count
            ),
            format!(
                "{label}.selected_route_nonbudget_drift_posture_count:{}",
                self.selected_route_nonbudget_drift_posture_count
            ),
        ]
    }

    pub fn planned_frontier_breadth(&self) -> usize {
        self.planned_frontier_breadth
    }

    pub fn reported_execution_records_examined_count(&self) -> usize {
        self.reported_execution_records_examined_count
    }

    pub fn planned_serial_fallback_route_count(&self) -> usize {
        self.planned_serial_fallback_route_count
    }

    pub fn bundle_serial_route_count(&self) -> usize {
        self.bundle_serial_route_count
    }

    pub fn planned_packet_merge_boundary_count(&self) -> usize {
        self.planned_packet_merge_boundary_count
    }

    pub fn selected_route_nonbudget_drift_posture_count(&self) -> usize {
        self.selected_route_nonbudget_drift_posture_count
    }
}
