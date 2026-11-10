use crate::application::{
    WorthQueryApplicationFacade, WorthQueryConfigSectionFamily, WorthQuerySupportReport,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;
use crate::projection_consumption::{
    certify_projection_consumption_closeout_core, ProjectionConsumptionCertificationBundle,
    ProjectionConsumptionCertifiedSourceSurface,
};
use crate::runtime::{
    WorthQueryAuthorityLane, WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport,
    WorthQueryRuntimeSupportProfile,
};

pub fn application_support_report() -> WorthQuerySupportReport {
    WorthQueryApplicationFacade::runtime_backed_default()
        .support_report()
        .expect("fixture digest must fit")
}

pub fn application_default_bootstrap_digest(report: &WorthQuerySupportReport) -> String {
    hash_parts(&[
        "milestone_nine_five_application_default_bootstrap_v1".to_string(),
        format!("report:{}", report.report_digest()),
        format!("matrix:{}", report.support_matrix().support_matrix_digest()),
        format!(
            "sections:{}",
            report
                .section_postures()
                .iter()
                .filter(|posture| posture.section() != WorthQueryConfigSectionFamily::Store)
                .map(|posture| format!(
                    "{}:{}:{}",
                    posture.section().as_str(),
                    posture.owner().as_str(),
                    posture.enabled()
                ))
                .collect::<Vec<_>>()
                .join("|")
        ),
    ])
}

pub fn public_bridge_bootstrap_support_digest() -> String {
    runtime_support_profile_digest(&public_bridge_runtime_support_profile())
}

pub fn public_bridge_bootstrap_contract_digest() -> String {
    hash_parts(&[
        "milestone_nine_five_public_bridge_bootstrap_v1".to_string(),
        format!("support:{}", public_bridge_bootstrap_support_digest()),
        "WorthQueryRuntime::builder".to_string(),
        "runtime_bridge".to_string(),
        "schema_adapter".to_string(),
        "source_adapter".to_string(),
        "existing_truth_verification".to_string(),
        "write_authority".to_string(),
        "signal_sink".to_string(),
        "subscription_activation".to_string(),
        "preview_basis".to_string(),
        "inspector_evidence".to_string(),
        "support_profile".to_string(),
        "build_backend_from_parts".to_string(),
    ])
}

pub fn projection_bundle(
) -> Result<ProjectionConsumptionCertificationBundle, IdentityDerivationDenial> {
    certify_projection_consumption_closeout_core()
}

pub fn projection_surface_digest(
    bundle: &ProjectionConsumptionCertificationBundle,
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> String {
    let inventory_digest = bundle
        .family_inventory()
        .rows()
        .iter()
        .find(
            |row: &&crate::projection_consumption::ProjectionConsumptionFamilyInventoryRow| {
                row.certified_surface() == surface
            },
        )
        .map(|row| row.row_digest().to_string())
        .expect("projection inventory must cover the requested certified surface");
    let support_rows = bundle
        .support_matrix()
        .rows()
        .iter()
        .filter(
            |row: &&crate::projection_consumption::ProjectionConsumptionSupportMatrixRow| {
                row.certified_surface() == surface
            },
        )
        .map(|row| row.row_digest().to_string())
        .collect::<Vec<_>>();
    hash_parts(
        &[vec![
            "milestone_nine_five_projection_surface_v1".to_string(),
            format!("surface:{}", surface.as_str()),
            format!("inventory:{inventory_digest}"),
            format!("support:{}", bundle.support_matrix().matrix_digest()),
            format!("oracle:{}", bundle.oracle_report().oracle_digest()),
            format!(
                "public_boundary:{}",
                bundle.public_boundary_audit().audit_digest()
            ),
        ]]
        .into_iter()
        .flatten()
        .chain(support_rows)
        .collect::<Vec<_>>(),
    )
}

fn public_bridge_runtime_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "test-subscription-activation",
        "test-preview-basis",
        "test-inspector-evidence",
    )
    .with_family_support(WorthQueryRuntimeFamilySupport::supported(
        WorthQueryRuntimeFacadeFamily::Write,
        [WorthQueryAuthorityLane::AuthoritativeTruth],
        [],
        ["test-write-authority"],
    ))
    .with_bridge_backed_verification_support(
        "probe_existing",
        "direct_entity_identity",
        true,
        true,
        None,
    )
    .with_bridge_backed_verification_support(
        "probe_existing",
        "direct_relation_identity",
        true,
        true,
        None,
    )
}

fn runtime_support_profile_digest(profile: &WorthQueryRuntimeSupportProfile) -> String {
    hash_parts(
        &[vec![format!("posture:{}", profile.posture().as_str())]]
            .into_iter()
            .flatten()
            .chain(profile.rows().map(|row| {
                format!(
                    "{}:{}:{}:{}:{}:{}",
                    row.family().as_str(),
                    row.status().as_str(),
                    row.teaching_posture().as_str(),
                    row.authority_lanes()
                        .iter()
                        .map(|lane| lane.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    row.effect_policies()
                        .iter()
                        .map(|policy| policy.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    row.evidence().join(","),
                )
            }))
            .collect::<Vec<_>>(),
    )
}
