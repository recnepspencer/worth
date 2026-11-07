use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::projection_consumption::ProjectionConsumptionCertifiedSourceSurface;
use crate::saved_query::{SavedQueryArtifact, SavedQueryReuseOutcome};

use super::super::axes::{
    MilestoneNineFiveBootstrapAxis, MilestoneNineFiveCompositionAxis,
    MilestoneNineFiveProjectionAxis, MilestoneNineFiveReuseAxis, MilestoneNineFiveViewAxis,
};
use super::super::digests::{
    application_default_bootstrap_digest, application_support_report, projection_bundle,
    public_bridge_bootstrap_contract_digest, public_bridge_bootstrap_support_digest,
};
use super::super::row::MilestoneNineFiveHostileLaneBundle;

pub fn projection_axis_for(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> MilestoneNineFiveProjectionAxis {
    match surface {
        ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding => {
            MilestoneNineFiveProjectionAxis::RetainedDerivedArtifactBinding
        }
        ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding => {
            MilestoneNineFiveProjectionAxis::LiveArtifactBinding
        }
        ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection => {
            MilestoneNineFiveProjectionAxis::RelationalGroupedProjection
        }
        _ => panic!("unexpected projection surface for milestone 9.5 hostile matrix"),
    }
}

pub fn bootstrap_axis_for(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> MilestoneNineFiveBootstrapAxis {
    match surface {
        ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding => {
            MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract
        }
        ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding
        | ProjectionConsumptionCertifiedSourceSurface::RelationalGroupedProjection => {
            MilestoneNineFiveBootstrapAxis::RuntimeBackedDefaultFacade
        }
        _ => MilestoneNineFiveBootstrapAxis::RuntimeBackedDefaultFacade,
    }
}

pub fn lane_bundle(
    composition_axis: MilestoneNineFiveCompositionAxis,
    view_axis: MilestoneNineFiveViewAxis,
    projection_axis: MilestoneNineFiveProjectionAxis,
    reuse_axis: MilestoneNineFiveReuseAxis,
    bootstrap_axis: MilestoneNineFiveBootstrapAxis,
    canonical_query_digest: String,
    canonical_result_shape_digest: String,
    composition_authority_digest: String,
    view_shape_digest: String,
    view_plan_digest: String,
    projection_contract_digest: String,
    saved: &SavedQueryArtifact,
    reuse: &SavedQueryReuseOutcome,
) -> Result<MilestoneNineFiveHostileLaneBundle, IdentityDerivationDenial> {
    let report = application_support_report();
    let projection_bundle = projection_bundle()?;
    let (reuse_matrix_digest, temporal_async_surface_posture) = match reuse {
        SavedQueryReuseOutcome::Admitted(decision) => (
            decision.matrix().digest().to_string(),
            decision
                .temporal_async_surface_posture()
                .as_str()
                .to_string(),
        ),
        SavedQueryReuseOutcome::Denied(denial) => (
            denial.matrix().digest().to_string(),
            denial.temporal_async_surface_posture().as_str().to_string(),
        ),
    };
    let (bootstrap_contract_digest, bootstrap_support_digest) = match bootstrap_axis {
        MilestoneNineFiveBootstrapAxis::RuntimeBackedDefaultFacade => (
            application_default_bootstrap_digest(&report),
            report.support_matrix().support_matrix_digest().to_string(),
        ),
        MilestoneNineFiveBootstrapAxis::PublicBridgeReadBootstrapContract => (
            public_bridge_bootstrap_contract_digest(),
            public_bridge_bootstrap_support_digest(),
        ),
    };
    let (admitted_families, _deferred_families, statuses) =
        crate::view_shape::runtime_backed_view_shape_support_profile();

    Ok(MilestoneNineFiveHostileLaneBundle {
        composition_axis: composition_axis.as_str().to_string(),
        view_axis: view_axis.as_str().to_string(),
        projection_axis: projection_axis.as_str().to_string(),
        reuse_axis: reuse_axis.as_str().to_string(),
        bootstrap_axis: bootstrap_axis.as_str().to_string(),
        canonical_query_digest,
        canonical_result_shape_digest,
        composition_authority_digest,
        composition_support_digest: report
            .query_composition_support_profile()
            .expect("runtime-backed default must publish query composition support")
            .profile_digest()
            .to_string(),
        view_shape_digest,
        view_plan_digest,
        view_support_digest: digest_parts(&[
            format!(
                "admitted:{}",
                admitted_families
                    .iter()
                    .map(|family| family.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "statuses:{}",
                statuses
                    .iter()
                    .map(|(family, status)| format!("{}:{}", family.as_str(), status.as_str()))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]),
        projection_contract_digest,
        projection_support_digest: projection_bundle
            .support_matrix()
            .matrix_digest()
            .to_string(),
        projection_oracle_digest: projection_bundle
            .oracle_report()
            .oracle_digest()
            .to_string(),
        saved_query_digest: saved.digest().as_str().to_string(),
        reuse_matrix_digest,
        temporal_async_surface_posture,
        bootstrap_contract_digest,
        bootstrap_support_digest,
        application_support_report_digest: report.report_digest().to_string(),
    })
}
