mod lanes;
mod model;
mod rejections;
mod row_catalog;
mod rows;

#[cfg(test)]
mod tests;

pub(crate) use model::{
    FrontierCertificationLane, FrontierCertificationMatrix, FrontierCertificationRejection,
    FrontierFailureClass, FrontierPerturbationClass, FrontierRouteClass,
    MilestoneFivePointThreeFrontierCertificationArtifact,
};
pub(crate) use row_catalog::{FRONTIER_CANONICAL_ROW_SPECS, FRONTIER_REJECTION_ROW_SPECS};

use self::lanes::{serial_control_lane, serial_fallback_bundle_lane, serial_fallback_lane};
use self::rows::{canonical_row, rejection_row};

pub struct MilestoneFivePointThreeFrontierCertificationAdapter;

impl MilestoneFivePointThreeFrontierCertificationAdapter {
    pub fn frontier_planning_and_serial_fallback_parity_test() -> FrontierCertificationMatrix {
        let serial_control = serial_control_lane();
        let serial_fallback = serial_fallback_lane();
        let bundle_lane = serial_fallback_bundle_lane();

        FrontierCertificationMatrix {
            suite_name: "Frontier Planning And Serial Fallback Parity Test",
            rows: FRONTIER_CANONICAL_ROW_SPECS
                .iter()
                .map(|spec| canonical_row(spec, &serial_control, &serial_fallback, &bundle_lane))
                .collect(),
            rejection_rows: FRONTIER_REJECTION_ROW_SPECS
                .iter()
                .map(|spec| rejection_row(spec, &serial_control))
                .collect(),
        }
    }

    pub fn frontier_planning_and_serial_fallback_parity_artifact(
    ) -> MilestoneFivePointThreeFrontierCertificationArtifact {
        Self::frontier_planning_and_serial_fallback_parity_test()
            .into_milestone_five_point_three_artifact()
    }
}
