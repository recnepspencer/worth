use super::super::FrontierPredictionDriftOutcome;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FrontierRouteCounters {
    route_serial_fallback_count: usize,
    route_nonbudget_drift_posture_count: usize,
}

impl FrontierRouteCounters {
    pub fn route_serial_fallback_count(&self) -> usize {
        self.route_serial_fallback_count
    }

    pub fn route_nonbudget_drift_posture_count(&self) -> usize {
        self.route_nonbudget_drift_posture_count
    }

    pub(in crate::frontier_planning::testing) fn serial(
        drift_outcome: &FrontierPredictionDriftOutcome,
    ) -> Self {
        Self {
            route_serial_fallback_count: 1,
            route_nonbudget_drift_posture_count: usize::from(
                *drift_outcome != FrontierPredictionDriftOutcome::WithinBudget,
            ),
        }
    }
}
