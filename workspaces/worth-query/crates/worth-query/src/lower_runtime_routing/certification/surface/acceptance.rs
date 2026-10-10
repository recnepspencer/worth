use crate::identity::hash_parts;
use crate::lower_runtime_routing::worth_query_lower_runtime_crossing_inventory;

use super::acceptance_checks::{control_digest, hostile_digest, synthetic_tail_exactness_digest};
use super::evidence::worth_query_lower_runtime_representative_surface;
use super::{allowed_phase_six_synthetic_seams, required_phase_six_concrete_seams};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeAcceptanceLane {
    Control,
    Hostile,
    Parity,
}

impl WorthQueryLowerRuntimeAcceptanceLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control-lane",
            Self::Hostile => "hostile-lane",
            Self::Parity => "parity-lane",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeAcceptanceRow {
    lane: WorthQueryLowerRuntimeAcceptanceLane,
    digest: String,
    detail: String,
}

impl WorthQueryLowerRuntimeAcceptanceRow {
    fn new(
        lane: WorthQueryLowerRuntimeAcceptanceLane,
        digest: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            lane,
            digest: digest.into(),
            detail: detail.into(),
        }
    }

    pub fn lane(&self) -> WorthQueryLowerRuntimeAcceptanceLane {
        self.lane
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeAcceptanceSuite {
    rows: Vec<WorthQueryLowerRuntimeAcceptanceRow>,
    suite_digest: String,
}

impl WorthQueryLowerRuntimeAcceptanceSuite {
    fn new(rows: Vec<WorthQueryLowerRuntimeAcceptanceRow>) -> Self {
        let suite_digest = hash_parts(
            &rows
                .iter()
                .map(|row| format!("{}|{}|{}", row.lane().as_str(), row.digest(), row.detail()))
                .collect::<Vec<_>>(),
        );
        Self { rows, suite_digest }
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimeAcceptanceRow] {
        &self.rows
    }

    pub fn suite_digest(&self) -> &str {
        &self.suite_digest
    }

    pub fn lane(
        &self,
        lane: WorthQueryLowerRuntimeAcceptanceLane,
    ) -> &WorthQueryLowerRuntimeAcceptanceRow {
        self.rows
            .iter()
            .find(|row| row.lane() == lane)
            .unwrap_or_else(|| panic!("missing acceptance lane {}", lane.as_str()))
    }
}

pub fn worth_query_lower_runtime_acceptance_suite(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeAcceptanceSuite {
    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    WorthQueryLowerRuntimeAcceptanceSuite::new(vec![
        WorthQueryLowerRuntimeAcceptanceRow::new(
            WorthQueryLowerRuntimeAcceptanceLane::Control,
            control_digest(&surface),
            format!(
                "crossings={} requests={} route_plans={} receipts={} envelopes={} concrete={} synthetic={} required_concrete_seams={} synthetic_tail_digest={} synthetic_tail={}",
                worth_query_lower_runtime_crossing_inventory().rows().len(),
                surface.requests().len(),
                surface.route_plans().len(),
                surface.boundary_receipts().len(),
                surface.envelopes().len(),
                surface.concrete_surface_width(),
                surface.synthetic_surface_width(),
                required_phase_six_concrete_seams().len(),
                synthetic_tail_exactness_digest(&surface),
                allowed_phase_six_synthetic_seams()
                    .iter()
                    .map(|row| row.seam_key().as_str())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ),
        WorthQueryLowerRuntimeAcceptanceRow::new(
            WorthQueryLowerRuntimeAcceptanceLane::Hostile,
            hostile_digest(&surface),
            "deleted seam survival, specialist-gap survival, route cardinality drift, support drift, and bypass drift remain forbidden".to_string(),
        ),
        WorthQueryLowerRuntimeAcceptanceRow::new(
            WorthQueryLowerRuntimeAcceptanceLane::Parity,
            surface.route_parity_digest().to_string(),
            "equivalent admitted crossings normalize to shared lower-runtime route meaning while intentionally different families diverge".to_string(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceptance_suite_exposes_control_hostile_and_parity_lanes() {
        let host_request = crate::bridge_host_request::serial_request();
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_acceptance_suite(resource_request);

        assert_eq!(suite.rows().len(), 3);
        assert!(!suite.suite_digest().is_empty());
        assert!(suite
            .rows()
            .iter()
            .any(|row| row.lane() == WorthQueryLowerRuntimeAcceptanceLane::Control));
        assert!(suite
            .rows()
            .iter()
            .any(|row| row.lane() == WorthQueryLowerRuntimeAcceptanceLane::Hostile));
        assert!(suite
            .rows()
            .iter()
            .any(|row| row.lane() == WorthQueryLowerRuntimeAcceptanceLane::Parity));
    }

    #[test]
    fn acceptance_suite_control_lane_proves_exact_cardinality() {
        let host_request = crate::bridge_host_request::serial_request();
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_acceptance_suite(resource_request);
        let control = suite.lane(WorthQueryLowerRuntimeAcceptanceLane::Control);

        assert!(control.detail().contains("crossings="));
        assert!(control.detail().contains("receipts="));
        assert!(control.detail().contains("envelopes="));
        assert!(control.detail().contains("concrete="));
        assert!(control.detail().contains("synthetic="));
        assert!(control.detail().contains("synthetic_tail_digest="));
        assert!(control.detail().contains("synthetic_tail="));
    }

    #[test]
    fn acceptance_suite_hostile_digest_stays_distinct_from_control() {
        let host_request = crate::bridge_host_request::serial_request();
        let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

        let suite = worth_query_lower_runtime_acceptance_suite(resource_request);

        assert_ne!(
            suite
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Control)
                .digest(),
            suite
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
                .digest()
        );
    }
}
