mod lanes;
mod model;
mod rejections;
mod requirements;
mod row_catalog;
mod rows;

#[cfg(test)]
mod tests;

pub(crate) use model::{
    closeout_matrix_digest_parts, FrontierCertificationLane, FrontierCertificationMatrix,
    FrontierCertificationRejection, FrontierCloseoutRequirement, FrontierCloseoutStatus,
    FrontierFailureClass, FrontierPerturbationClass, FrontierRouteClass,
    MilestoneFivePointThreeFrontierCertificationArtifact,
    MilestoneFivePointThreeFrontierCloseoutArtifact,
};
pub(crate) use row_catalog::{
    FRONTIER_CANONICAL_ROW_SPECS, FRONTIER_REJECTION_ROW_SPECS,
    FRONTIER_REQUIRED_CANONICAL_ROW_NAMES, FRONTIER_REQUIRED_REJECTION_ROW_NAMES,
};

use self::lanes::{serial_control_lane, serial_fallback_bundle_lane, serial_fallback_lane};
use self::requirements::{
    acceptance_evidence_requirements, must_preserve_requirements, must_ship_requirements,
    proof_obligation_requirements,
};
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

    pub fn frontier_planning_closeout_artifact() -> MilestoneFivePointThreeFrontierCloseoutArtifact
    {
        let certification = Self::frontier_planning_and_serial_fallback_parity_artifact();
        let must_ship = must_ship_requirements();
        let must_preserve = must_preserve_requirements();
        let proof_obligations = proof_obligation_requirements();
        let acceptance_evidence = acceptance_evidence_requirements();
        let closeout_matrix_digest =
            crate::harness::certification::digest_parts(&closeout_matrix_digest_parts(
                &[
                    ("must_ship", &must_ship),
                    ("must_preserve", &must_preserve),
                    ("proof_obligations", &proof_obligations),
                    ("acceptance_evidence", &acceptance_evidence),
                ],
                &certification.certification_bundle_digest,
            ));

        MilestoneFivePointThreeFrontierCloseoutArtifact {
            suite_name: certification.suite_name,
            closeout_matrix_digest,
            certification_bundle_digest: certification.certification_bundle_digest,
            must_ship,
            must_preserve,
            proof_obligations,
            acceptance_evidence,
        }
    }
}
