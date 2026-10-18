use super::expectations::assert_matrix_meets_required_rows;
use super::model::{
    MilestoneFivePointTwoPreviewCertificationArtifact, PreviewCertificationLane,
    PreviewCertificationMatrix,
};
use super::MilestoneFivePointTwoPreviewCertificationAdapter;
use crate::preview::{
    PreviewBindingCounters, PreviewComparisonCounters, PreviewExecutionCounters,
    PreviewLiveCounters,
};

#[test]
fn preview_certification_matrix_meets_required_rows() {
    let matrix =
        MilestoneFivePointTwoPreviewCertificationAdapter::preview_session_basis_and_promotion_parity_test();

    assert_matrix_meets_required_rows(&matrix);
}

#[test]
fn preview_certification_artifact_reports_offline_ready_completeness() {
    let artifact = MilestoneFivePointTwoPreviewCertificationAdapter::
        preview_session_basis_and_promotion_parity_artifact();

    assert_artifact_expectations(&artifact);
}

#[derive(Default)]
struct ExpectedPreviewCounterSnapshots {
    binding: PreviewBindingCounters,
    execution: PreviewExecutionCounters,
    comparison: PreviewComparisonCounters,
    preview_live: PreviewLiveCounters,
}

impl ExpectedPreviewCounterSnapshots {
    fn from_matrix(matrix: &PreviewCertificationMatrix) -> Self {
        let mut expected = Self::default();
        for lane in matrix
            .rows
            .iter()
            .flat_map(|row| [&row.control_lane, &row.hostile_lane, &row.parity_lane])
            .chain(
                matrix
                    .rejection_rows
                    .iter()
                    .flat_map(|row| [&row.control_lane, &row.parity_lane]),
            )
        {
            expected.absorb_lane(lane);
        }
        expected.absorb_hostile_rejection_counters(matrix);
        expected
    }

    fn absorb_lane(&mut self, lane: &PreviewCertificationLane) {
        self.binding.absorb(&lane.counters);
        self.execution.absorb(&lane.execution_counters);
        if let Some(counters) = lane.comparison_counters.as_ref() {
            self.comparison.absorb(counters);
        }
        if let Some(counters) = lane.preview_live_counters.as_ref() {
            self.preview_live.absorb(counters);
        }
    }

    fn absorb_hostile_rejection_counters(&mut self, matrix: &PreviewCertificationMatrix) {
        for counters in matrix
            .rejection_rows
            .iter()
            .filter_map(|row| row.hostile_lane.counters.as_ref())
        {
            self.binding.absorb(counters);
        }
        for counters in matrix
            .rejection_rows
            .iter()
            .filter_map(|row| row.hostile_lane.execution_counters.as_ref())
        {
            self.execution.absorb(counters);
        }
        for counters in matrix
            .rejection_rows
            .iter()
            .filter_map(|row| row.hostile_lane.comparison_counters.as_ref())
        {
            self.comparison.absorb(counters);
        }
        for counters in matrix
            .rejection_rows
            .iter()
            .filter_map(|row| row.hostile_lane.preview_live_counters.as_ref())
        {
            self.preview_live.absorb(counters);
        }
    }
}

fn assert_artifact_expectations(artifact: &MilestoneFivePointTwoPreviewCertificationArtifact) {
    let expected = ExpectedPreviewCounterSnapshots::from_matrix(&artifact.matrix);

    assert_artifact_metadata_and_completeness(artifact);
    assert_preview_live_snapshot(artifact, &expected.preview_live);
    assert_binding_snapshot(artifact, &expected.binding);
    assert_workflow_and_comparison_snapshots(artifact, &expected.execution, &expected.comparison);
    assert_linkage_snapshots(artifact, &expected.binding);
    assert_remaining_execution_snapshots(artifact, &expected.execution);
}

fn assert_artifact_metadata_and_completeness(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
) {
    assert!(
        artifact
            .bundle_completeness_report
            .covers_all_currently_implemented_normative_scenarios
    );
    assert_eq!(
        artifact.bundle_completeness_report.offline_analysis_ready,
        artifact
            .bundle_completeness_report
            .covers_full_milestone_five_point_two_spec_matrix
    );
    assert_eq!(
        artifact
            .bundle_completeness_report
            .zero_rediscovery_lane_count,
        artifact.bundle_completeness_report.supported_lane_count
    );
    assert!(
        artifact
            .bundle_completeness_report
            .preview_live_composition_admitted_by_design
    );
}

fn assert_preview_live_snapshot(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
    expected: &PreviewLiveCounters,
) {
    assert!(
        expected.preview_live_admission_count() > 0,
        "artifact counter snapshot should retain preview-live admissions"
    );
    assert_eq!(
        artifact
            .preview_live_counter_snapshot
            .preview_live_admission_count(),
        expected.preview_live_admission_count()
    );
    assert!(
        expected.preview_live_execution_count() > 0,
        "artifact counter snapshot should retain preview-live execution counts"
    );
    assert_eq!(
        artifact
            .preview_live_counter_snapshot
            .preview_live_execution_count(),
        expected.preview_live_execution_count()
    );
    assert!(
        expected.preview_live_drift_denial_count() > 0,
        "artifact counter snapshot should retain preview-live drift denials"
    );
    assert_eq!(
        artifact
            .preview_live_counter_snapshot
            .preview_live_drift_denial_count(),
        expected.preview_live_drift_denial_count()
    );
    assert!(
        expected.preview_live_rebind_available_count() > 0,
        "artifact counter snapshot should retain preview-live explicit rebinds"
    );
    assert_eq!(
        artifact
            .preview_live_counter_snapshot
            .preview_live_rebind_available_count(),
        expected.preview_live_rebind_available_count()
    );
    assert!(
        expected.preview_live_broad_fallback_denial_count() > 0,
        "artifact counter snapshot should retain preview-live broad-fallback denials"
    );
    assert_eq!(
        artifact
            .preview_live_counter_snapshot
            .preview_live_broad_fallback_denial_count(),
        expected.preview_live_broad_fallback_denial_count()
    );
}

fn assert_binding_snapshot(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
    expected: &PreviewBindingCounters,
) {
    assert!(
        expected.preview_invalid_basis_denial_count() > 0,
        "artifact counter snapshot should retain hostile invalid-basis denials"
    );
    assert_eq!(
        artifact
            .binding_counter_snapshot
            .preview_invalid_basis_denial_count(),
        expected.preview_invalid_basis_denial_count()
    );
    assert!(
        expected.preview_broad_fallback_denial_count() > 0,
        "artifact counter snapshot should retain hostile broad-fallback denials"
    );
    assert_eq!(
        artifact
            .binding_counter_snapshot
            .preview_broad_fallback_denial_count(),
        expected.preview_broad_fallback_denial_count()
    );
    assert!(
        expected.preview_invalid_lifecycle_denial_count() > 0,
        "artifact counter snapshot should retain hostile stale-lifecycle denials"
    );
    assert_eq!(
        artifact
            .binding_counter_snapshot
            .preview_invalid_lifecycle_denial_count(),
        expected.preview_invalid_lifecycle_denial_count()
    );
}

fn assert_workflow_and_comparison_snapshots(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
    expected_execution: &PreviewExecutionCounters,
    expected_comparison: &PreviewComparisonCounters,
) {
    assert!(expected_execution.preview_workflow_foundation_artifact_lookup_count() > 0);
    assert_eq!(
        artifact
            .execution_counter_snapshot
            .preview_workflow_foundation_artifact_lookup_count(),
        expected_execution.preview_workflow_foundation_artifact_lookup_count()
    );
    assert_eq!(
        artifact
            .execution_counter_snapshot
            .preview_workflow_foundation_admission_count(),
        expected_execution.preview_workflow_foundation_admission_count()
    );
    assert!(expected_comparison.preview_promotion_comparison_count() > 0);
    assert_eq!(
        artifact
            .comparison_counter_snapshot
            .preview_promotion_comparison_count(),
        expected_comparison.preview_promotion_comparison_count()
    );
}

fn assert_linkage_snapshots(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
    expected: &PreviewBindingCounters,
) {
    assert!(
        expected.preview_replay_bundle_lookup_count() > 0,
        "artifact counter snapshot should retain replay-linkage lookups"
    );
    assert_eq!(
        artifact
            .binding_counter_snapshot
            .preview_replay_bundle_lookup_count(),
        expected.preview_replay_bundle_lookup_count()
    );
    assert!(
        expected.preview_bridge_promotion_linkage_count() > 0,
        "artifact counter snapshot should retain promotion-linkage lookups"
    );
    assert_eq!(
        artifact
            .binding_counter_snapshot
            .preview_bridge_promotion_linkage_count(),
        expected.preview_bridge_promotion_linkage_count()
    );
}

fn assert_remaining_execution_snapshots(
    artifact: &MilestoneFivePointTwoPreviewCertificationArtifact,
    expected: &PreviewExecutionCounters,
) {
    assert_eq!(
        artifact
            .execution_counter_snapshot
            .preview_work_avoided_by_explicit_basis_count(),
        expected.preview_work_avoided_by_explicit_basis_count()
    );
    assert_eq!(
        artifact
            .execution_counter_snapshot
            .preview_workflow_foundation_denial_count(),
        expected.preview_workflow_foundation_denial_count()
    );
}
