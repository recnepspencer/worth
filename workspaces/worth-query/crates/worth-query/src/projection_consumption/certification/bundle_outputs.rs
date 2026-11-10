use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_closeout_forbidden_fallback_surface_row_digest,
    compose_closeout_oracle_surface_row_digest, compose_closeout_proof_shape_surface_row_digest,
    compose_closeout_public_boundary_surface_row_digest,
    compose_closeout_seeded_replay_surface_row_digest,
    compose_closeout_support_matrix_surface_row_digest, compose_failure_digest_bundle,
    compose_negative_dx_boundary_digest, compose_target_dx_digest,
};

use super::super::receipt_transitions::ProjectionConsumptionTransitionRules;
use super::audits::{
    ProjectionConsumptionFamilyInventory, ProjectionConsumptionForbiddenFallbackAudit,
    ProjectionConsumptionProofShapeAudit, ProjectionConsumptionPublicBoundaryAudit,
    ProjectionConsumptionSupportMatrix,
};
use super::bundle::{
    ProjectionConsumptionCertificationLane, ProjectionConsumptionCertificationRow,
};
use super::fixtures::{
    denied_masked_field_failure_digest, source_digest, source_mismatch_failure_digest,
    source_receipt_digest, ProjectionConsumptionCertifiedLifecycle,
};
use super::oracle::ProjectionConsumptionOracleReport;
use super::seeded::ProjectionConsumptionSeededCertificationReport;
use super::slopes::ProjectionConsumptionSlopeReport;

pub struct ProjectionConsumptionBundleOutputs {
    pub rows: Vec<ProjectionConsumptionCertificationRow>,
    pub outputs: Vec<(&'static str, String)>,
}

pub fn assemble_closeout_bundle_outputs(
    lifecycle: &ProjectionConsumptionCertifiedLifecycle,
    family_inventory: &ProjectionConsumptionFamilyInventory,
    support_matrix: &ProjectionConsumptionSupportMatrix,
    public_boundary_audit: &ProjectionConsumptionPublicBoundaryAudit,
    proof_shape_audit: &ProjectionConsumptionProofShapeAudit,
    forbidden_fallback_audit: &ProjectionConsumptionForbiddenFallbackAudit,
    oracle_report: &ProjectionConsumptionOracleReport,
    seeded_report: &ProjectionConsumptionSeededCertificationReport,
    slope_report: &ProjectionConsumptionSlopeReport,
) -> Result<ProjectionConsumptionBundleOutputs, IdentityDerivationDenial> {
    let transition_rules = ProjectionConsumptionTransitionRules::current_phase_five_surface()?;
    let target_dx_digest = target_dx_digest()?;
    let negative_dx_boundary_digest = compose_negative_dx_boundary_digest(
        public_boundary_audit.audit_digest(),
        proof_shape_audit.proof_shape_digest(),
    )?;
    let failure_digest = compose_failure_digest_bundle(
        &denied_masked_field_failure_digest()?,
        &source_mismatch_failure_digest()?,
    )?;
    let rows = vec![
        certification_row(
            ProjectionConsumptionCertificationLane::SupportMatrixSurface,
            format!(
                "inventory:{}|matrix:{}|traceability:{}",
                family_inventory.inventory_digest(),
                support_matrix.matrix_digest(),
                support_matrix.support_traceability_digest()
            ),
            compose_closeout_support_matrix_surface_row_digest(
                family_inventory.inventory_digest(),
                support_matrix.matrix_digest(),
                support_matrix.support_traceability_digest(),
            )?,
        ),
        certification_row(
            ProjectionConsumptionCertificationLane::PublicBoundarySurface,
            format!(
                "public_surface:{}|negative_dx:{}",
                public_boundary_audit.audit_digest(),
                negative_dx_boundary_digest
            ),
            compose_closeout_public_boundary_surface_row_digest(
                public_boundary_audit.audit_digest(),
                negative_dx_boundary_digest.as_str(),
            )?,
        ),
        certification_row(
            ProjectionConsumptionCertificationLane::ProofShapeSurface,
            format!(
                "proof_shape:{}|phase_progression:{}",
                proof_shape_audit.proof_shape_digest(),
                proof_shape_audit.phase_progression_digest()
            ),
            compose_closeout_proof_shape_surface_row_digest(
                proof_shape_audit.proof_shape_digest(),
                proof_shape_audit.phase_progression_digest(),
            )?,
        ),
        certification_row(
            ProjectionConsumptionCertificationLane::ForbiddenFallbackSurface,
            format!(
                "forbidden_fallback:{}|total_occurrences:{}",
                forbidden_fallback_audit.audit_digest(),
                forbidden_fallback_audit.total_occurrence_count()
            ),
            compose_closeout_forbidden_fallback_surface_row_digest(
                forbidden_fallback_audit.audit_digest(),
                forbidden_fallback_audit.total_occurrence_count(),
            )?,
        ),
        certification_row(
            ProjectionConsumptionCertificationLane::OracleSurface,
            format!(
                "oracle:{}|manifest:{}",
                oracle_report.oracle_digest(),
                oracle_report.manifest_digest()
            ),
            compose_closeout_oracle_surface_row_digest(
                oracle_report.oracle_digest(),
                oracle_report.manifest_digest(),
            )?,
        ),
        certification_row(
            ProjectionConsumptionCertificationLane::SeededReplaySurface,
            format!(
                "seeded:{}|replay:{}|classes:{}",
                seeded_report.seeded_sequence_digest(),
                seeded_report.seed_replay_digest(),
                seeded_report.seed_generator_class_digest()
            ),
            compose_closeout_seeded_replay_surface_row_digest(
                seeded_report.seeded_sequence_digest(),
                seeded_report.seed_replay_digest(),
                seeded_report.seed_generator_class_digest(),
            )?,
        ),
    ];
    let outputs = vec![
        (
            "query_digest",
            lifecycle
                .declaration()
                .binding()
                .authorized_projection_query_digest()
                .to_string(),
        ),
        (
            "result_shape_digest",
            lifecycle
                .declaration()
                .binding()
                .result_shape_digest()
                .to_string(),
        ),
        (
            "authorized_projection_digest",
            lifecycle
                .declaration()
                .binding()
                .authorized_projection_identity()
                .to_string(),
        ),
        (
            "materialization_basis_digest",
            lifecycle
                .contract()
                .basis_digest()
                .unwrap_or("none")
                .to_string(),
        ),
        (
            "projection_consumption_declaration_digest",
            lifecycle.declaration().declaration_digest().to_string(),
        ),
        (
            "projection_consumption_eligibility_digest",
            lifecycle.contract().eligibility_digest().to_string(),
        ),
        (
            "materialized_projection_contract_digest",
            lifecycle.contract().contract_digest().to_string(),
        ),
        (
            "consumed_projection_fact_set_digest",
            lifecycle.facts().fact_set_digest().to_string(),
        ),
        (
            "projection_consumption_receipt_digest",
            lifecycle.receipt().receipt_digest().to_string(),
        ),
        (
            "projection_consumption_envelope_digest",
            lifecycle.envelope().envelope_digest().to_string(),
        ),
        (
            "projection_source_digest",
            source_digest(lifecycle.contract())?,
        ),
        (
            "projection_source_receipt_digest",
            source_receipt_digest(lifecycle.contract())?,
        ),
        (
            "projection_fact_family_inventory_digest",
            family_inventory.inventory_digest().to_string(),
        ),
        (
            "projection_support_matrix_digest",
            support_matrix.matrix_digest().to_string(),
        ),
        (
            "projection_public_surface_digest",
            public_boundary_audit.audit_digest().to_string(),
        ),
        ("projection_target_dx_digest", target_dx_digest),
        (
            "projection_proof_shape_digest",
            proof_shape_audit.proof_shape_digest().to_string(),
        ),
        (
            "projection_forbidden_fallback_digest",
            forbidden_fallback_audit.audit_digest().to_string(),
        ),
        (
            "projection_forbidden_fallback_total_occurrences",
            forbidden_fallback_audit
                .total_occurrence_count()
                .to_string(),
        ),
        (
            "projection_phase_progression_digest",
            proof_shape_audit.phase_progression_digest().to_string(),
        ),
        (
            "projection_transition_rules_digest",
            transition_rules.rules_digest().to_string(),
        ),
        (
            "projection_oracle_digest",
            oracle_report.oracle_digest().to_string(),
        ),
        (
            "projection_oracle_manifest_digest",
            oracle_report.manifest_digest().to_string(),
        ),
        (
            "projection_support_traceability_digest",
            support_matrix.support_traceability_digest().to_string(),
        ),
        (
            "seeded_sequence_digest",
            seeded_report.seeded_sequence_digest().to_string(),
        ),
        (
            "seed_replay_digest",
            seeded_report.seed_replay_digest().to_string(),
        ),
        (
            "seed_generator_class_digest",
            seeded_report.seed_generator_class_digest().to_string(),
        ),
        ("negative_dx_boundary_digest", negative_dx_boundary_digest),
        ("failure_digest", failure_digest),
        (
            "counter_snapshot",
            slope_report.counter_snapshot().digest().to_string(),
        ),
        (
            "authority_reopen_count",
            slope_report
                .counter_snapshot()
                .authority_reopen_count()
                .to_string(),
        ),
        (
            "fact_extraction_width",
            slope_report
                .counter_snapshot()
                .source_row_width_consumed()
                .to_string(),
        ),
        (
            "projection_declaration_slope_digest",
            slope_report.declaration_slope_digest().to_string(),
        ),
        (
            "projection_eligibility_slope_digest",
            slope_report.eligibility_slope_digest().to_string(),
        ),
        (
            "projection_contract_binding_slope_digest",
            slope_report.contract_binding_slope_digest().to_string(),
        ),
        (
            "projection_fact_extraction_slope_digest",
            slope_report.fact_extraction_slope_digest().to_string(),
        ),
        (
            "projection_receipt_materialization_slope_digest",
            slope_report
                .receipt_materialization_slope_digest()
                .to_string(),
        ),
        (
            "projection_envelope_materialization_slope_digest",
            slope_report
                .envelope_materialization_slope_digest()
                .to_string(),
        ),
        (
            "projection_support_lookup_slope_digest",
            slope_report.support_lookup_slope_digest().to_string(),
        ),
    ];
    Ok(ProjectionConsumptionBundleOutputs { rows, outputs })
}

fn certification_row(
    lane: ProjectionConsumptionCertificationLane,
    evidence_detail: String,
    row_digest: String,
) -> ProjectionConsumptionCertificationRow {
    ProjectionConsumptionCertificationRow {
        lane,
        evidence_detail,
        row_digest,
    }
}

fn target_dx_digest() -> Result<String, IdentityDerivationDenial> {
    compose_target_dx_digest()
}
