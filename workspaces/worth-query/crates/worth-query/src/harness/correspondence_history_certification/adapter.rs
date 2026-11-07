use crate::harness::certification::{
    milestone_five_point_four_requirements, unmet_required_assertion_classes, unmet_required_rows,
    CertificationMatrix, RequiredAssertionClass,
};

use super::fixtures::{self, CertificationLanes};
use super::model::{
    CorrespondenceHistoryBundleCompletenessReport, CorrespondenceHistoryCertificationMatrix,
    MilestoneFivePointFourCorrespondenceHistoryCertificationArtifact,
};
use super::row_catalog::{
    CORRESPONDENCE_HISTORY_CANONICAL_ROW_SPECS, CORRESPONDENCE_HISTORY_REJECTION_ROW_SPECS,
};
use super::rows::{canonical_row, rejection_row};

pub struct MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter;

impl MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter {
    pub fn structural_correspondence_and_historical_materialization_path_test(
    ) -> CorrespondenceHistoryCertificationMatrix {
        let lanes = CertificationLanes {
            lineage: fixtures::lineage_authoritative_lane().expect("fixture digest must fit"),
            structural: fixtures::structural_unique_replay_lane().expect("fixture digest must fit"),
            disagreement: fixtures::disagreement_lane().expect("fixture digest must fit"),
            ambiguity: fixtures::ambiguity_lane().expect("fixture digest must fit"),
            retained: fixtures::retained_lane().expect("fixture digest must fit"),
            replay: fixtures::replay_lane().expect("fixture digest must fit"),
            reconstruction: fixtures::reconstruction_lane().expect("fixture digest must fit"),
            drift: fixtures::prediction_drift_lane().expect("fixture digest must fit"),
        };

        CertificationMatrix {
            suite_name: "Structural Correspondence And Historical Materialization Path Test",
            rows: CORRESPONDENCE_HISTORY_CANONICAL_ROW_SPECS
                .iter()
                .map(|spec| canonical_row(spec, &lanes))
                .collect(),
            rejection_rows: CORRESPONDENCE_HISTORY_REJECTION_ROW_SPECS
                .iter()
                .map(|spec| rejection_row(spec, &lanes))
                .collect(),
        }
    }

    pub fn structural_correspondence_and_historical_materialization_path_artifact(
    ) -> MilestoneFivePointFourCorrespondenceHistoryCertificationArtifact {
        let matrix = Self::structural_correspondence_and_historical_materialization_path_test();
        let requirements = milestone_five_point_four_requirements();
        let all_lanes_emit_required_outputs = matrix.rows.iter().all(|row| {
            row.control_lane.has_required_outputs()
                && row.hostile_lane.has_required_outputs()
                && row.parity_lane.has_required_outputs()
        }) && matrix
            .rejection_rows
            .iter()
            .all(|row| row.hostile_lane.has_required_outputs());
        let zero_rediscovery_lane_count = matrix
            .rows
            .iter()
            .flat_map(|row| [&row.control_lane, &row.hostile_lane, &row.parity_lane])
            .filter(|lane| lane.has_zero_rediscovery())
            .count();
        let supported_lane_count = matrix.rows.len() * 3;
        let completeness = CorrespondenceHistoryBundleCompletenessReport {
            canonical_row_count: matrix.rows.len(),
            rejection_row_count: matrix.rejection_rows.len(),
            all_lanes_emit_required_outputs,
            zero_rediscovery_lane_count,
            unmet_required_rows: unmet_required_rows(
                &matrix,
                requirements.required_canonical_rows,
                requirements.required_rejection_rows,
            ),
            unmet_required_assertion_classes: unmet_required_assertion_classes(
                &[
                    RequiredAssertionClass::Equality,
                    RequiredAssertionClass::Inequality,
                    RequiredAssertionClass::TypedFailure,
                    RequiredAssertionClass::ZeroResidue,
                ],
                requirements.required_assertion_classes,
            ),
            offline_analysis_ready: all_lanes_emit_required_outputs
                && zero_rediscovery_lane_count == supported_lane_count,
        };

        matrix.into_milestone_five_point_four_artifact(completeness)
    }
}
