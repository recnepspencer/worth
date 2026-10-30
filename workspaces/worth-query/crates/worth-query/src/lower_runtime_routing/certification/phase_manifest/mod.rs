use crate::identity::hash_parts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimePhaseArtifact {
    CrossingInventory,
    CapabilityRequest,
    CapabilityEligibility,
    RoutePlanOrReadmissionHandoff,
    BoundaryExecutionReceipt,
    BoundaryEnvelope,
    SupportMatrix,
    CloseoutRegistry,
    PublicSurfaceInventory,
    BoundaryReconciliationReport,
    NonBypassAudit,
    ProofShapeAudit,
    PerformanceSlopeReport,
    AcceptanceSuite,
    CertificationBundle,
    NamedClosureTest,
    StabilizationCloseoutReport,
}

impl WorthQueryLowerRuntimePhaseArtifact {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CrossingInventory => "lower_runtime_crossing_inventory",
            Self::CapabilityRequest => "lower_runtime_capability_request",
            Self::CapabilityEligibility => "lower_runtime_capability_eligibility",
            Self::RoutePlanOrReadmissionHandoff => {
                "lower_runtime_route_plan_or_readmission_handoff"
            }
            Self::BoundaryExecutionReceipt => "lower_runtime_boundary_execution_receipt",
            Self::BoundaryEnvelope => "lower_runtime_boundary_envelope",
            Self::SupportMatrix => "lower_runtime_support_matrix",
            Self::CloseoutRegistry => "lower_runtime_closeout_registry",
            Self::PublicSurfaceInventory => "lower_runtime_public_surface_inventory",
            Self::BoundaryReconciliationReport => "lower_runtime_boundary_reconciliation_report",
            Self::NonBypassAudit => "lower_runtime_non_bypass_audit",
            Self::ProofShapeAudit => "lower_runtime_proof_shape_audit",
            Self::PerformanceSlopeReport => "lower_runtime_performance_slope_report",
            Self::AcceptanceSuite => "lower_runtime_acceptance_suite",
            Self::CertificationBundle => "lower_runtime_certification_bundle",
            Self::NamedClosureTest => "lower_runtime_named_closure_test",
            Self::StabilizationCloseoutReport => "lower_runtime_stabilization_closeout_report",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimePhaseManifestRow {
    artifact: WorthQueryLowerRuntimePhaseArtifact,
    producer: &'static str,
    required_input: &'static str,
    next_consumer: &'static str,
    enforcement_proof: &'static str,
    row_digest: String,
}

impl WorthQueryLowerRuntimePhaseManifestRow {
    fn new(
        artifact: WorthQueryLowerRuntimePhaseArtifact,
        producer: &'static str,
        required_input: &'static str,
        next_consumer: &'static str,
        enforcement_proof: &'static str,
    ) -> Self {
        let row_digest = hash_parts(&[
            "lower_runtime_routing_phase_manifest_row_v1".to_string(),
            format!("artifact:{}", artifact.as_str()),
            format!("producer:{producer}"),
            format!("required_input:{required_input}"),
            format!("next_consumer:{next_consumer}"),
            format!("proof:{enforcement_proof}"),
        ]);
        Self {
            artifact,
            producer,
            required_input,
            next_consumer,
            enforcement_proof,
            row_digest,
        }
    }

    pub fn artifact(&self) -> WorthQueryLowerRuntimePhaseArtifact {
        self.artifact
    }

    pub fn producer(&self) -> &'static str {
        self.producer
    }

    pub fn required_input(&self) -> &'static str {
        self.required_input
    }

    pub fn next_consumer(&self) -> &'static str {
        self.next_consumer
    }

    pub fn enforcement_proof(&self) -> &'static str {
        self.enforcement_proof
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimePhaseManifest {
    rows: Vec<WorthQueryLowerRuntimePhaseManifestRow>,
    manifest_digest: String,
    typestate_transition_digest: String,
}

impl WorthQueryLowerRuntimePhaseManifest {
    fn new(rows: Vec<WorthQueryLowerRuntimePhaseManifestRow>) -> Self {
        let manifest_digest = hash_parts(
            &rows
                .iter()
                .map(|row| row.row_digest().to_string())
                .collect::<Vec<_>>(),
        );
        let typestate_transition_digest = hash_parts(
            &rows
                .windows(2)
                .map(|pair| {
                    format!(
                        "{}->{}|{}|{}",
                        pair[0].artifact().as_str(),
                        pair[1].artifact().as_str(),
                        pair[0].row_digest(),
                        pair[1].row_digest()
                    )
                })
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            manifest_digest,
            typestate_transition_digest,
        }
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimePhaseManifestRow] {
        &self.rows
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn typestate_transition_digest(&self) -> &str {
        &self.typestate_transition_digest
    }
}

pub fn worth_query_lower_runtime_phase_manifest() -> WorthQueryLowerRuntimePhaseManifest {
    use WorthQueryLowerRuntimePhaseArtifact::*;

    WorthQueryLowerRuntimePhaseManifest::new(vec![
        row(
            CrossingInventory,
            "worth_query_lower_runtime_crossing_inventory",
            "locked covered seam inventory from 9.3.5 plus surviving specialist classifications",
            "WorthQueryLowerRuntimeCapabilityRequest::new",
            "worth_query_lower_runtime_crossing_inventory",
        ),
        row(
            CapabilityRequest,
            "WorthQueryLowerRuntimeCapabilityRequest::new",
            "Crossing inventory row plus seam/route/authority subject digest",
            "WorthQueryLowerRuntimeCapabilityEligibility::{admitted,deferred,unsupported,forbidden}",
            "capability_request_constructor_private",
        ),
        row(
            CapabilityEligibility,
            "WorthQueryLowerRuntimeCapabilityEligibility::{admitted,deferred,unsupported,forbidden}",
            "LowerRuntimeCapabilityRequest",
            "WorthQueryLowerRuntimeRoutePlan::new or WorthQueryLowerRuntimeReadmissionReceipt::new",
            "certify_lower_runtime_routing",
        ),
        row(
            RoutePlanOrReadmissionHandoff,
            "WorthQueryLowerRuntimeRoutePlan::new / WorthQueryLowerRuntimeReadmissionReceipt::new",
            "CapabilityEligibility",
            "WorthQueryLowerRuntimeBoundaryExecutionReceipt::{from_route_plan,from_readmission_receipt}",
            "route_plan_constructor_private",
        ),
        row(
            BoundaryExecutionReceipt,
            "WorthQueryLowerRuntimeBoundaryExecutionReceipt::{from_route_plan,from_readmission_receipt}",
            "LowerRuntimeRoutePlan or LowerRuntimeReadmissionReceipt",
            "WorthQueryLowerRuntimeBoundaryEnvelope::{from_route_plan,from_readmission_receipt}",
            "boundary_execution_receipt_constructor_private",
        ),
        row(
            BoundaryEnvelope,
            "WorthQueryLowerRuntimeBoundaryEnvelope::{from_route_plan,from_readmission_receipt}",
            "BoundaryExecutionReceipt plus retained authority evidence",
            "worth_query_lower_runtime_support_matrix / inspect_lower_runtime_boundary",
            "boundary_envelope_constructor_private",
        ),
        row(
            SupportMatrix,
            "worth_query_lower_runtime_support_matrix",
            "BoundaryEnvelope plus admitted/deferred/eliminated support posture binding",
            "worth_query_lower_runtime_closeout_registry / certify_lower_runtime_routing",
            "support_matrix_rows_cover_crossings_and_closeout_registry",
        ),
        row(
            CloseoutRegistry,
            "worth_query_lower_runtime_closeout_registry",
            "support matrix plus eliminated/deferred seam rows",
            "worth_query_lower_runtime_boundary_reconciliation_report / worth_query_lower_runtime_acceptance_suite",
            "closeout_registry_has_no_compatibility_debt_and_names_deferred_neighbors_explicitly",
        ),
        row(
            PublicSurfaceInventory,
            "worth_query_lower_runtime_public_surface_inventory",
            "crossing inventory plus direct-import audit mirror",
            "certify_lower_runtime_non_bypass / worth_query_lower_runtime_boundary_reconciliation_report",
            "public_surface_rows_reference_known_routed_or_audited_seams",
        ),
        row(
            BoundaryReconciliationReport,
            "worth_query_lower_runtime_boundary_reconciliation_report",
            "public surface inventory plus direct-import audit posture",
            "certify_lower_runtime_routing",
            "allowed_boundary_adapters_stay_audit_backed_inside_reconciliation_report",
        ),
        row(
            NonBypassAudit,
            "certify_lower_runtime_non_bypass",
            "public surface inventory plus allowed boundary scan targets",
            "certify_lower_runtime_routing",
            "certify_lower_runtime_non_bypass",
        ),
        row(
            ProofShapeAudit,
            "worth_query_lower_runtime_proof_shape_audit",
            "compile-fail boundary plus certification enforcement lanes",
            "certify_lower_runtime_routing",
            "proof_shape_audit_tracks_phase_progression",
        ),
        row(
            PerformanceSlopeReport,
            "certify_lower_runtime_performance_slopes",
            "representative surface plus closeout/deferred width counters",
            "certify_lower_runtime_routing",
            "slope_report_emits_all_outputs_from_observed_profiles",
        ),
        row(
            AcceptanceSuite,
            "worth_query_lower_runtime_acceptance_suite",
            "representative surface plus support/closeout agreement and hostile parity checks",
            "certify_lower_runtime_routing",
            "worth_query_lower_runtime_acceptance_suite",
        ),
        row(
            CertificationBundle,
            "certify_lower_runtime_routing",
            "acceptance suite, boundary reconciliation, non-bypass audit, proof-shape audit, and performance slope report",
            "worth_query_lower_runtime_closure_test",
            "phase_manifest_is_public_and_consumable_by_closeout_bundle",
        ),
        row(
            NamedClosureTest,
            "worth_query_lower_runtime_closure_test",
            "certification bundle plus acceptance suite, downstream boundary audit, and compile-boundary evidence",
            "worth_query_lower_runtime_closeout_report",
            "closure_test_binds_boundary_and_compile_lanes_to_certified_rows",
        ),
        row(
            StabilizationCloseoutReport,
            "worth_query_lower_runtime_closeout_report",
            "named closure test plus certification bundle, phase manifest, acceptance suite, boundary reconciliation, and synthetic tail report",
            "runtime-api-public-stabilization gate",
            "closeout_report_keeps_stabilization_inputs_in_sync",
        ),
    ])
}

pub fn worth_query_lower_runtime_phase_artifact_manifest_digest() -> String {
    worth_query_lower_runtime_phase_manifest()
        .manifest_digest()
        .to_string()
}

pub fn worth_query_lower_runtime_typestate_transition_digest() -> String {
    worth_query_lower_runtime_phase_manifest()
        .typestate_transition_digest()
        .to_string()
}

fn row(
    artifact: WorthQueryLowerRuntimePhaseArtifact,
    producer: &'static str,
    required_input: &'static str,
    next_consumer: &'static str,
    enforcement_proof: &'static str,
) -> WorthQueryLowerRuntimePhaseManifestRow {
    WorthQueryLowerRuntimePhaseManifestRow::new(
        artifact,
        producer,
        required_input,
        next_consumer,
        enforcement_proof,
    )
}

#[cfg(test)]
mod tests;
