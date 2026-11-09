use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{HistoricalCapabilityDescriptor, HistoricalPathReuseDescriptor};
use crate::harness::certification::{
    CanonicalCertificationRow, ParityAnchor, RejectionCertificationRow,
};
use crate::query_context::{
    bind_diff_query_context, bind_legacy_query_basis_context, reject_raw_storage_delta_access,
    QueryBasisContextRequest, QueryContextBindingSource,
};

use super::super::lane::{
    HistoricalDiffLane, HistoricalDiffPerturbationClass, HistoricalDiffRejection,
};
use super::super::row_catalog::{HistoricalDiffCanonicalRowSpec, HistoricalDiffRejectionRowSpec};
use super::lanes::{
    branch_diff_lane, branch_lane, current_historical_diff_lane, current_lane, historical_lane,
    preview_lane, store_historical_lane,
};
use super::rejections::{
    basis_substitution_error, comparison_broadening_error, comparison_shape_mismatch_error,
    diff_scope_mismatch_error, historical_broadening_error, preview_lane_context,
};

pub(super) fn canonical_row(
    spec: &HistoricalDiffCanonicalRowSpec,
    current: &HistoricalDiffLane,
    branch: &HistoricalDiffLane,
    historical: &HistoricalDiffLane,
    store_historical: &HistoricalDiffLane,
    preview: &HistoricalDiffLane,
    branch_diff: &HistoricalDiffLane,
    current_historical_diff: &HistoricalDiffLane,
) -> Result<
    CanonicalCertificationRow<HistoricalDiffPerturbationClass, HistoricalDiffLane>,
    IdentityDerivationDenial,
> {
    let (control_lane, hostile_lane) = match spec.row_name {
        "current-vs-branch-basis-explicitness" => (current.clone(), branch.clone()),
        "current-vs-historical-basis-explicitness" => (current.clone(), historical.clone()),
        "historical-materialization-path-explicitness" => (current.clone(), historical.clone()),
        "runtime-vs-store-historical-parity" => (historical.clone(), store_historical.clone()),
        "diff-comparison-family-explicitness" => (branch_diff.clone(), branch.clone()),
        "branch-to-branch-diff-shaped" => (branch.clone(), branch_diff.clone()),
        "current-to-historical-diff-shaped" => {
            (historical.clone(), current_historical_diff.clone())
        }
        "result-shape-parity-across-basis-variants" => (current.clone(), historical.clone()),
        "preview-derived-historical-basis-explicitness" => (historical.clone(), preview.clone()),
        "admitted-diff-cost-class-explicitness" => (branch.clone(), branch_diff.clone()),
        "prediction-versus-realization-explicitness" => (branch.clone(), branch_diff.clone()),
        other => panic!("unexpected historical diff canonical row {other}"),
    };
    let parity_lane = match spec.row_name {
        "current-vs-branch-basis-explicitness" => current_lane(),
        "current-vs-historical-basis-explicitness" => current_lane(),
        "historical-materialization-path-explicitness" => historical_lane()?,
        "runtime-vs-store-historical-parity" => store_historical_lane()?,
        "diff-comparison-family-explicitness" => branch_diff_lane(),
        "branch-to-branch-diff-shaped" => branch_diff_lane(),
        "current-to-historical-diff-shaped" => current_historical_diff_lane()?,
        "result-shape-parity-across-basis-variants" => current_lane(),
        "preview-derived-historical-basis-explicitness" => preview_lane(),
        "admitted-diff-cost-class-explicitness" => branch_diff_lane(),
        "prediction-versus-realization-explicitness" => branch_diff_lane(),
        other => panic!("unexpected historical diff canonical row {other}"),
    };

    Ok(CanonicalCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        hostile_expectation: spec.hostile_expectation,
        parity_anchor: ParityAnchor::Control,
        control_lane,
        hostile_lane,
        parity_lane,
    })
}

pub(super) fn rejection_row(
    spec: &HistoricalDiffRejectionRowSpec,
) -> Result<
    RejectionCertificationRow<
        HistoricalDiffPerturbationClass,
        HistoricalDiffLane,
        HistoricalDiffRejection,
    >,
    IdentityDerivationDenial,
> {
    let control_lane = current_lane();
    let parity_lane = branch_lane();
    let hostile_lane = match spec.row_name {
        "unsupported-historical-basis" => HistoricalDiffRejection::from_error(
            &bind_legacy_query_basis_context(
                QueryBasisContextRequest::historical_snapshot("history:unsupported"),
                QueryContextBindingSource::HistoricalCapability(
                    &HistoricalCapabilityDescriptor::retained_snapshot_for_test(
                        "history:unsupported",
                        HistoricalPathReuseDescriptor::retained_reuse(),
                    )?,
                ),
            )
            .expect_err("raw historical capability should not mint an admitted query context"),
        ),
        "ambiguous-comparison-basis" => {
            let preview = preview_lane_context();
            HistoricalDiffRejection::from_error(
                &bind_diff_query_context(&preview, &preview)
                    .expect_err("preview to preview comparison should stay ambiguous"),
            )
        }
        "diff-scope-mismatch" => HistoricalDiffRejection::from_error(&diff_scope_mismatch_error()),
        "forbidden-basis-substitution" => {
            HistoricalDiffRejection::from_error(&basis_substitution_error()?)
        }
        "raw-storage-delta-leakage-forbidden" => {
            HistoricalDiffRejection::from_error(&reject_raw_storage_delta_access())
        }
        "historical-broadening-denied" => {
            HistoricalDiffRejection::from_error(&historical_broadening_error()?)
        }
        "broadening-required-comparison-denial" => {
            HistoricalDiffRejection::from_error(&comparison_broadening_error())
        }
        "declared-result-shape-mismatch" => {
            HistoricalDiffRejection::from_error(&comparison_shape_mismatch_error())
        }
        other => panic!("unexpected historical diff rejection row {other}"),
    };

    Ok(RejectionCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        control_lane,
        hostile_lane,
        parity_lane,
    })
}
