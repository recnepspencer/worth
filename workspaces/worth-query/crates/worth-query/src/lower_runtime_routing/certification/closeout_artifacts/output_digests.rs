use crate::identity::hash_parts;
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_closeout_registry, worth_query_lower_runtime_crossing_inventory,
    worth_query_lower_runtime_gap_registry, worth_query_lower_runtime_support_matrix,
};

use super::bundle_types::WorthQueryLowerRuntimeCertificationOutputDigest;
use crate::lower_runtime_routing::certification::performance::WorthQueryLowerRuntimePerformanceSlopeReport;
use crate::lower_runtime_routing::certification::surface::{
    allowed_phase_six_synthetic_seams, worth_query_lower_runtime_synthetic_tail_report,
    WorthQueryLowerRuntimeAcceptanceSuite, WorthQueryLowerRuntimeRepresentativeSurface,
};
use crate::lower_runtime_routing::certification::{
    worth_query_lower_runtime_boundary_reconciliation_report,
    worth_query_lower_runtime_phase_artifact_manifest_digest,
    worth_query_lower_runtime_phase_progression_digest,
    worth_query_lower_runtime_proof_shape_digest,
    worth_query_lower_runtime_typestate_transition_digest, WorthQueryLowerRuntimeCertificationRow,
    WorthQueryLowerRuntimeNonBypassAudit,
};

pub(super) fn certification_output_digests(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
    _acceptance: &WorthQueryLowerRuntimeAcceptanceSuite,
    non_bypass: &WorthQueryLowerRuntimeNonBypassAudit,
    slopes: &WorthQueryLowerRuntimePerformanceSlopeReport,
    certification_rows: &[WorthQueryLowerRuntimeCertificationRow],
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<WorthQueryLowerRuntimeCertificationOutputDigest> {
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let gaps = worth_query_lower_runtime_gap_registry();
    let closeout = worth_query_lower_runtime_closeout_registry();
    let support = worth_query_lower_runtime_support_matrix();

    vec![
        output("query_digest", surface.query_digest().to_string()),
        output("capability_request_digest", digest_requests(surface)),
        output("capability_family_digest", digest_capability_families()),
        output(
            "capability_eligibility_digest",
            digest_eligibilities(surface),
        ),
        output(
            "lower_runtime_route_plan_digest",
            digest_route_plans(surface),
        ),
        output(
            "boundary_execution_receipt_digest",
            digest_boundary_receipts(surface),
        ),
        output(
            "lower_runtime_boundary_envelope_digest",
            digest_envelopes(surface),
        ),
        output(
            "crossing_inventory_digest",
            crossings
                .inventory_digest()
                .terminal_projection_for_reporting()
                .to_string(),
        ),
        output(
            "crossing_classification_digest",
            crossings
                .classification_digest()
                .terminal_projection_for_reporting()
                .to_string(),
        ),
        output("compatibility_debt_registry_digest", gaps.registry_digest()),
        output(
            "debt_exit_criteria_digest",
            hash_parts(&[
                gaps.debt_exit_criteria_digest(),
                closeout.required_closeout_digest(),
            ]),
        ),
        output(
            "route_authority_digest",
            digest_envelope_field(surface, "authority"),
        ),
        output(
            "route_evidence_digest",
            digest_envelope_field(surface, "evidence"),
        ),
        output(
            "route_cost_posture_digest",
            digest_envelope_field(surface, "cost"),
        ),
        output(
            "route_failure_topology_digest",
            digest_envelope_field(surface, "failure"),
        ),
        output("route_support_matrix_digest", support.matrix_digest()),
        output(
            "route_public_surface_digest",
            non_bypass.route_public_surface_digest().to_string(),
        ),
        output(
            "route_boundary_reconciliation_digest",
            worth_query_lower_runtime_boundary_reconciliation_report()
                .report_digest()
                .to_string(),
        ),
        output(
            "route_concrete_surface_digest",
            surface.concrete_surface_digest(),
        ),
        output(
            "route_phase_artifact_manifest_digest",
            worth_query_lower_runtime_phase_artifact_manifest_digest(),
        ),
        output(
            "route_synthetic_surface_digest",
            surface.synthetic_surface_digest(),
        ),
        output(
            "route_synthetic_tail_policy_digest",
            digest_synthetic_tail_policy(),
        ),
        output(
            "route_synthetic_tail_report_digest",
            worth_query_lower_runtime_synthetic_tail_report(resource_request)
                .report_digest()
                .to_string(),
        ),
        output(
            "route_synthetic_tail_justification_digest",
            worth_query_lower_runtime_synthetic_tail_report(resource_request)
                .justification_digest()
                .to_string(),
        ),
        output(
            "route_proof_shape_digest",
            worth_query_lower_runtime_proof_shape_digest(),
        ),
        output(
            "route_phase_progression_digest",
            worth_query_lower_runtime_phase_progression_digest(),
        ),
        output(
            "route_typestate_transition_digest",
            worth_query_lower_runtime_typestate_transition_digest(),
        ),
        output(
            "route_parity_digest",
            surface.route_parity_digest().to_string(),
        ),
        output(
            "route_non_bypass_digest",
            non_bypass.route_non_bypass_digest().to_string(),
        ),
        output("lower_runtime_gap_registry_digest", gaps.registry_digest()),
        output(
            "failure_digest",
            hash_parts(
                &certification_rows
                    .iter()
                    .filter_map(|row| row.failure_digest().map(str::to_string))
                    .collect::<Vec<_>>(),
            ),
        ),
        output(
            "counter_snapshot",
            slopes
                .full_profile()
                .counters()
                .counter_snapshot_digest()
                .to_string(),
        ),
        output(
            "crossing_inventory_width",
            slopes
                .full_profile()
                .counters()
                .crossing_inventory_width()
                .to_string(),
        ),
        output(
            "compatibility_debt_width",
            slopes
                .full_profile()
                .counters()
                .compatibility_debt_width()
                .to_string(),
        ),
        output(
            "route_plan_width",
            slopes
                .full_profile()
                .counters()
                .route_plan_width()
                .to_string(),
        ),
        output(
            "boundary_evidence_width",
            slopes
                .full_profile()
                .counters()
                .boundary_evidence_width()
                .to_string(),
        ),
        output(
            "route_concrete_surface_width",
            surface.concrete_surface_width().to_string(),
        ),
        output(
            "route_synthetic_surface_width",
            surface.synthetic_surface_width().to_string(),
        ),
        output(
            "route_boundary_reconciliation_width",
            worth_query_lower_runtime_boundary_reconciliation_report()
                .rows()
                .len()
                .to_string(),
        ),
        output(
            "route_synthetic_tail_width",
            worth_query_lower_runtime_synthetic_tail_report(resource_request)
                .rows()
                .len()
                .to_string(),
        ),
        output(
            "capability_eligibility_slope_digest",
            slope_digest(slopes, "capability_eligibility_slope_digest"),
        ),
        output(
            "route_plan_assembly_slope_digest",
            slope_digest(slopes, "route_plan_assembly_slope_digest"),
        ),
        output(
            "boundary_receipt_assembly_slope_digest",
            slope_digest(slopes, "boundary_receipt_assembly_slope_digest"),
        ),
        output(
            "boundary_envelope_assembly_slope_digest",
            slope_digest(slopes, "boundary_envelope_assembly_slope_digest"),
        ),
        output(
            "support_lookup_slope_digest",
            slope_digest(slopes, "support_lookup_slope_digest"),
        ),
        output(
            "debt_registry_lookup_slope_digest",
            slope_digest(slopes, "debt_registry_lookup_slope_digest"),
        ),
    ]
}

fn output(name: &'static str, digest: String) -> WorthQueryLowerRuntimeCertificationOutputDigest {
    WorthQueryLowerRuntimeCertificationOutputDigest::new(name, digest)
}

fn digest_requests(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(
        &surface
            .requests()
            .iter()
            .map(|request| request.request_digest().to_string())
            .collect::<Vec<_>>(),
    )
}

fn digest_eligibilities(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(
        &surface
            .eligibilities()
            .iter()
            .map(|eligibility| {
                eligibility
                    .eligibility_identity()
                    .terminal_projection_for_reporting()
                    .to_string()
            })
            .collect::<Vec<_>>(),
    )
}

fn digest_route_plans(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(
        &surface
            .route_plans()
            .iter()
            .map(|plan| plan.route_digest().to_string())
            .collect::<Vec<_>>(),
    )
}

fn digest_boundary_receipts(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(
        &surface
            .boundary_receipts()
            .iter()
            .map(|receipt| {
                receipt
                    .boundary_execution_identity()
                    .terminal_projection_for_reporting()
                    .to_string()
            })
            .collect::<Vec<_>>(),
    )
}

fn digest_envelopes(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(
        &surface
            .envelopes()
            .iter()
            .map(|envelope| {
                envelope
                    .envelope_identity()
                    .terminal_projection_for_reporting()
                    .to_string()
            })
            .collect::<Vec<_>>(),
    )
}

fn digest_capability_families() -> String {
    hash_parts(
        &worth_query_lower_runtime_crossing_inventory()
            .rows()
            .iter()
            .map(|row| row.capability_label().to_string())
            .collect::<Vec<_>>(),
    )
}

fn digest_synthetic_tail_policy() -> String {
    hash_parts(
        &allowed_phase_six_synthetic_seams()
            .iter()
            .map(|row| format!("{}|{}", row.seam_key().as_str(), row.justification()))
            .collect::<Vec<_>>(),
    )
}

fn digest_envelope_field(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
    field: &str,
) -> String {
    hash_parts(
        &surface
            .envelopes()
            .iter()
            .map(|envelope| match field {
                "authority" => envelope
                    .route_authority_identity()
                    .terminal_projection_for_reporting()
                    .to_string(),
                "evidence" => envelope
                    .route_evidence_identity()
                    .terminal_projection_for_reporting()
                    .to_string(),
                "cost" => envelope.route_cost_posture().as_str().to_string(),
                _ => envelope.route_failure_topology().as_str().to_string(),
            })
            .collect::<Vec<_>>(),
    )
}

fn slope_digest(
    report: &WorthQueryLowerRuntimePerformanceSlopeReport,
    output_name: &str,
) -> String {
    report
        .digest_for_output(output_name)
        .unwrap_or_else(|| panic!("missing slope digest {output_name}"))
        .to_string()
}
