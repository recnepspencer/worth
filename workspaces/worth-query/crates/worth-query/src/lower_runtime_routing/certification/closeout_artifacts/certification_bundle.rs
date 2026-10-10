use crate::identity::hash_parts;
use crate::lower_runtime_routing::{
    certify_lower_runtime_non_bypass, worth_query_lower_runtime_closeout_registry,
    worth_query_lower_runtime_crossing_inventory, worth_query_lower_runtime_gap_registry,
    worth_query_lower_runtime_support_matrix, WorthQueryLowerRuntimeCrossingClassification,
    WorthQueryLowerRuntimeSeamKey,
};

use super::bundle_types::{
    WorthQueryLowerRuntimeCertificationBundle, WorthQueryLowerRuntimeCertificationLane,
    WorthQueryLowerRuntimeCertificationRow,
};
use super::output_digests::certification_output_digests;
use crate::lower_runtime_routing::certification::boundary_certification::{
    worth_query_lower_runtime_boundary_reconciliation_report,
    worth_query_lower_runtime_proof_shape_audit,
};
use crate::lower_runtime_routing::certification::performance::certify_lower_runtime_performance_slopes;
use crate::lower_runtime_routing::certification::surface::{
    worth_query_lower_runtime_acceptance_suite, worth_query_lower_runtime_representative_surface,
    worth_query_lower_runtime_synthetic_tail_report, WorthQueryLowerRuntimeAcceptanceLane,
};

pub fn certify_lower_runtime_routing(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeCertificationBundle {
    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let acceptance = worth_query_lower_runtime_acceptance_suite(resource_request);
    let non_bypass = certify_lower_runtime_non_bypass()
        .expect("current workspace should satisfy the lower-runtime non-bypass audit");
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let closeout = worth_query_lower_runtime_closeout_registry();
    let support = worth_query_lower_runtime_support_matrix();
    let slopes = certify_lower_runtime_performance_slopes(&surface);
    let proof_shape = worth_query_lower_runtime_proof_shape_audit();
    let boundary_reconciliation = worth_query_lower_runtime_boundary_reconciliation_report();
    let synthetic_tail = worth_query_lower_runtime_synthetic_tail_report(resource_request);
    let rows = vec![
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::CrossingsSurface,
            crossings
                .inventory_digest()
                .terminal_projection_for_reporting()
                .to_string(),
            format!(
                "crossings={} support={} route_plans={} envelopes={}",
                crossings.rows().len(),
                support.rows().len(),
                surface.route_plans().len(),
                surface.envelopes().len()
            ),
            counter_digest(&[
                format!("crossings:{}", crossings.rows().len()),
                format!("support:{}", support.rows().len()),
                format!("route_plans:{}", surface.route_plans().len()),
                format!("envelopes:{}", surface.envelopes().len()),
            ]),
            None,
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::BoundaryClosureSurface,
            boundary_reconciliation.report_digest().to_string(),
            format!(
                "boundary_rows={} public_surface={} checked_files={}",
                boundary_reconciliation.rows().len(),
                non_bypass.route_public_surface_digest(),
                non_bypass.checked_file_count()
            ),
            counter_digest(&[
                format!("boundary_rows:{}", boundary_reconciliation.rows().len()),
                format!("checked_files:{}", non_bypass.checked_file_count()),
            ]),
            Some(non_bypass.route_non_bypass_digest().to_string()),
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::AcceptanceEvidence,
            acceptance.suite_digest().to_string(),
            acceptance
                .lane(WorthQueryLowerRuntimeAcceptanceLane::Control)
                .detail()
                .to_string(),
            counter_digest(&[
                format!("acceptance_lanes:{}", acceptance.rows().len()),
                format!("crossings:{}", crossings.rows().len()),
                format!("closeout_rows:{}", closeout.rows().len()),
            ]),
            Some(
                acceptance
                    .lane(WorthQueryLowerRuntimeAcceptanceLane::Hostile)
                    .digest()
                    .to_string(),
            ),
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::SyntheticTailPolicy,
            synthetic_tail.report_digest().to_string(),
            format!(
                "synthetic_tail_rows={} justification_digest={}",
                synthetic_tail.rows().len(),
                synthetic_tail.justification_digest()
            ),
            counter_digest(&[format!(
                "synthetic_tail_rows:{}",
                synthetic_tail.rows().len()
            )]),
            Some(
                acceptance
                    .lane(WorthQueryLowerRuntimeAcceptanceLane::Control)
                    .digest()
                    .to_string(),
            ),
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::RouteParity,
            surface.route_parity_digest().to_string(),
            "equivalent current-read and readmission routes normalize to shared authority/route posture".to_string(),
            counter_digest(&[
                "parity_pairs:2".to_string(),
                format!("route_plans:{}", surface.route_plans().len()),
            ]),
            None,
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::FormerSpecialistSeamClosure,
            former_specialist_closure_digest(),
            "frontier and bridge writeback specialist seams are adapter-classified and absent from the gap registry".to_string(),
            counter_digest(&["checked_seams:2".to_string()]),
            None,
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::DeferredNeighborDenial,
            closeout.registry_digest(),
            "deferred/store-temporal-async neighbors remain explicit closeout rows rather than admitted support".to_string(),
            counter_digest(&[
                format!("closeout_rows:{}", closeout.rows().len()),
                format!("gap_rows:{}", worth_query_lower_runtime_gap_registry().rows().len()),
            ]),
            Some(deferred_neighbor_failure_digest()),
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::DownstreamBoundaryAudit,
            non_bypass.route_non_bypass_digest().to_string(),
            format!(
                "checked_files={} delegated_public_surface={}",
                non_bypass.checked_file_count(),
                non_bypass.route_public_surface_digest()
            ),
            counter_digest(&[format!("checked_files:{}", non_bypass.checked_file_count())]),
            Some(downstream_boundary_failure_digest(&non_bypass)),
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::ProofShapeSurface,
            proof_shape.proof_shape_digest().to_string(),
            "proof-shape audit rejects bypass, omission, deferred masquerade, specialist debt survival, and downstream boundary leaks".to_string(),
            counter_digest(&[format!("proof_rows:{}", proof_shape.rows().len())]),
            None,
        ),
        certification_row(
            WorthQueryLowerRuntimeCertificationLane::Performance,
            hash_parts(
                &slopes
                    .rows()
                    .iter()
                    .map(|row| row.slope_digest().to_string())
                    .collect::<Vec<_>>(),
            ),
            format!(
                "profiles={} full_counter_snapshot={} route_width={} evidence_width={} deferred_width={}",
                slopes.profiles().len(),
                slopes.full_profile().counters().counter_snapshot_digest(),
                slopes.full_profile().counters().route_plan_width(),
                slopes.full_profile().counters().boundary_evidence_width(),
                slopes.full_profile().counters().deferred_width(),
            ),
            slopes
                .full_profile()
                .counters()
                .counter_snapshot_digest()
                .to_string(),
            None,
        ),
    ];
    let output_digests = certification_output_digests(
        &surface,
        &acceptance,
        &non_bypass,
        &slopes,
        &rows,
        resource_request,
    );

    WorthQueryLowerRuntimeCertificationBundle::new(rows, output_digests)
}

fn certification_row(
    lane: WorthQueryLowerRuntimeCertificationLane,
    artifact_digest: String,
    detail: String,
    counter_snapshot_digest: String,
    failure_digest: Option<String>,
) -> WorthQueryLowerRuntimeCertificationRow {
    WorthQueryLowerRuntimeCertificationRow::new(
        lane,
        artifact_digest,
        detail,
        counter_snapshot_digest,
        failure_digest,
    )
}

fn former_specialist_closure_digest() -> String {
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let gaps = worth_query_lower_runtime_gap_registry();
    let frontier = crossings
        .rows()
        .iter()
        .find(|row| row.seam_key() == WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake)
        .expect("frontier evidence seam should remain in the crossing inventory");
    let writeback = crossings
        .rows()
        .iter()
        .find(|row| row.seam_key() == WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback)
        .expect("writeback seam should remain in the crossing inventory");
    assert_eq!(
        frontier.classification(),
        WorthQueryLowerRuntimeCrossingClassification::QueryBoundaryAdapter
    );
    assert_eq!(
        writeback.classification(),
        WorthQueryLowerRuntimeCrossingClassification::QueryBoundaryAdapter
    );
    assert!(
        !gaps.rows().iter().any(|row| {
            matches!(
                row.seam_key(),
                WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake
                    | WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback
            )
        }),
        "former specialist seams must not survive in the gap registry"
    );
    hash_parts(&[frontier.row_digest(), writeback.row_digest()])
}

fn deferred_neighbor_failure_digest() -> String {
    hash_parts(
        &worth_query_lower_runtime_closeout_registry()
            .rows()
            .iter()
            .filter(|row| row.posture().as_str() == "deferred-neighbor")
            .map(|row| {
                format!(
                    "{}|{}|{}",
                    row.seam_key().as_str(),
                    row.closeout_target(),
                    row.required_closeout()
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn downstream_boundary_failure_digest(
    non_bypass: &crate::lower_runtime_routing::WorthQueryLowerRuntimeNonBypassAudit,
) -> String {
    hash_parts(&[
        "hostile_projection_file_outside_runtime_boundary_is_rejected".to_string(),
        non_bypass.route_public_surface_digest().to_string(),
    ])
}

fn counter_digest(parts: &[String]) -> String {
    hash_parts(parts)
}
