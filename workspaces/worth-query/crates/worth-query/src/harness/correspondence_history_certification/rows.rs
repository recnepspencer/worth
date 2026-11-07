use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::ParityAnchor;

use super::fixtures::{
    broad_candidate_scan_rejection, executor_path_mutation_rejection,
    hidden_materialization_substitution_rejection, host_cache_history_authority_rejection,
    unsupported_correspondence_family_rejection, unsupported_historical_materialization_rejection,
    CertificationLanes,
};
use super::model;
use super::row_catalog::{
    CorrespondenceHistoryCanonicalRowSpec, CorrespondenceHistoryRejectionRowSpec,
};

pub(crate) fn canonical_row(
    spec: &CorrespondenceHistoryCanonicalRowSpec,
    lanes: &CertificationLanes,
) -> model::CorrespondenceHistoryCertificationRow {
    let (control_lane, hostile_lane, parity_lane) = match spec.row_name {
        "lineage-correspondence-authoritative" | "lineage-correspondence-explicitness" => (
            lanes.lineage.clone(),
            lanes.lineage.clone(),
            lanes.lineage.clone(),
        ),
        "structural-correspondence-advisory" | "structural-correspondence-explicitness" => (
            lanes.structural.clone(),
            lanes.lineage.clone(),
            lanes.structural.clone(),
        ),
        "lineage-structural-disagreement-explicit" => (
            lanes.disagreement.clone(),
            lanes.disagreement.clone(),
            lanes.disagreement.clone(),
        ),
        "structural-ambiguity-explicit" | "correspondence-ambiguity-explicitness" => (
            lanes.ambiguity.clone(),
            lanes.ambiguity.clone(),
            lanes.ambiguity.clone(),
        ),
        "historical-retained-snapshot-path"
        | "retained-snapshot-path-explicitness"
        | "work-avoided-counter-parity" => (
            lanes.retained.clone(),
            lanes.retained.clone(),
            lanes.retained.clone(),
        ),
        "historical-delta-replay-path"
        | "delta-replay-path-explicitness"
        | "historical-cost-posture-parity" => (
            lanes.replay.clone(),
            lanes.retained.clone(),
            lanes.replay.clone(),
        ),
        "historical-full-reconstruction-path" | "full-reconstruction-path-explicitness" => (
            lanes.reconstruction.clone(),
            lanes.reconstruction.clone(),
            lanes.reconstruction.clone(),
        ),
        "historical-path-no-substitution" => (
            lanes.replay.clone(),
            lanes.replay.clone(),
            lanes.replay.clone(),
        ),
        "correspondence-cost-posture-parity" => (
            lanes.structural.clone(),
            lanes.lineage.clone(),
            lanes.structural.clone(),
        ),
        "prediction-drift-explicit" | "prediction-drift-explicitness" => (
            lanes.drift.clone(),
            lanes.drift.clone(),
            lanes.drift.clone(),
        ),
        other => panic!("unknown 5.4 canonical row {other}"),
    };

    model::CorrespondenceHistoryCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        hostile_expectation: spec.hostile_expectation,
        parity_anchor: ParityAnchor::Control,
        control_lane,
        hostile_lane,
        parity_lane,
    }
}

pub(crate) fn rejection_row(
    spec: &CorrespondenceHistoryRejectionRowSpec,
    lanes: &CertificationLanes,
) -> Result<model::CorrespondenceHistoryRejectionRow, IdentityDerivationDenial> {
    let (control_lane, hostile_lane, parity_lane) = match spec.row_name {
        "unsupported-correspondence-family" => (
            lanes.structural.clone(),
            unsupported_correspondence_family_rejection()?,
            lanes.structural.clone(),
        ),
        "unsupported-historical-materialization-path" => (
            lanes.replay.clone(),
            unsupported_historical_materialization_rejection()?,
            lanes.replay.clone(),
        ),
        "hidden-materialization-path-substitution-forbidden" => (
            lanes.replay.clone(),
            hidden_materialization_substitution_rejection()?,
            lanes.replay.clone(),
        ),
        "broad-candidate-scan-success-forbidden" => (
            lanes.structural.clone(),
            broad_candidate_scan_rejection()?,
            lanes.structural.clone(),
        ),
        "no-executor-path-mutation-after-planning" => (
            lanes.replay.clone(),
            executor_path_mutation_rejection()?,
            lanes.replay.clone(),
        ),
        "host-cache-history-authority-forbidden" => (
            lanes.replay.clone(),
            host_cache_history_authority_rejection()?,
            lanes.replay.clone(),
        ),
        other => panic!("unknown 5.4 rejection row {other}"),
    };

    Ok(model::CorrespondenceHistoryRejectionRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        control_lane,
        hostile_lane,
        parity_lane,
    })
}
