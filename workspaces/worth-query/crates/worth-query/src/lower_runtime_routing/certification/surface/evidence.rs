use std::collections::BTreeMap;

use crate::identity::hash_parts;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_crossing_inventory, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRoutePlan,
};

use super::fixtures::{
    hostile_parity_divergence_digest, normalized_parity_digest,
    representative_basis_subscription_readmission_row,
    representative_basis_truth_view_readmission_row,
    representative_causal_bridge_materialization_row, representative_compose_read_row,
    representative_effect_bridge_writeback_row, representative_effect_relational_merge_row,
    representative_effect_relational_mutation_row,
    representative_execute_read_family_in_basis_context_row,
    representative_execute_read_family_row, representative_frontier_evidence_row,
    representative_historical_bridge_lowering_row, representative_intent_runtime_execution_row,
    representative_live_view_schema_row, representative_live_view_source_row,
    representative_preview_basis_row, representative_projection_bridge_row,
    representative_projection_query_receipts_row, representative_projection_relational_row,
    representative_public_live_view_declaration_row,
    representative_runtime_basis_context_read_graph_row,
    representative_runtime_current_read_graph_row, representative_runtime_intent_authority_row,
    representative_runtime_live_installation_orchestration_row,
    representative_signal_invalidation_row, representative_subscription_activation_row,
    representative_subscription_continuity_row, representative_write_authority_row,
    synthetic_inventory_row, RepresentativeArtifacts,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeRepresentativeEvidenceSource {
    RuntimeBackedFixture,
    InventorySynthesized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeRepresentativeSurface {
    requests: Vec<WorthQueryLowerRuntimeCapabilityRequest>,
    eligibilities: Vec<WorthQueryLowerRuntimeCapabilityEligibility>,
    route_plans: Vec<WorthQueryLowerRuntimeRoutePlan>,
    boundary_receipts: Vec<WorthQueryLowerRuntimeBoundaryExecutionReceipt>,
    envelopes: Vec<WorthQueryLowerRuntimeBoundaryEnvelope>,
    query_digest: String,
    route_parity_digest: String,
    evidence_sources: BTreeMap<&'static str, WorthQueryLowerRuntimeRepresentativeEvidenceSource>,
    requests_by_seam: BTreeMap<&'static str, WorthQueryLowerRuntimeCapabilityRequest>,
    route_plans_by_seam: BTreeMap<&'static str, WorthQueryLowerRuntimeRoutePlan>,
    receipts_by_seam: BTreeMap<&'static str, WorthQueryLowerRuntimeBoundaryExecutionReceipt>,
    envelopes_by_seam: BTreeMap<&'static str, WorthQueryLowerRuntimeBoundaryEnvelope>,
}

impl WorthQueryLowerRuntimeRepresentativeSurface {
    pub fn requests(&self) -> &[WorthQueryLowerRuntimeCapabilityRequest] {
        &self.requests
    }

    pub fn eligibilities(&self) -> &[WorthQueryLowerRuntimeCapabilityEligibility] {
        &self.eligibilities
    }

    pub fn route_plans(&self) -> &[WorthQueryLowerRuntimeRoutePlan] {
        &self.route_plans
    }

    pub fn boundary_receipts(&self) -> &[WorthQueryLowerRuntimeBoundaryExecutionReceipt] {
        &self.boundary_receipts
    }

    pub fn envelopes(&self) -> &[WorthQueryLowerRuntimeBoundaryEnvelope] {
        &self.envelopes
    }

    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn route_parity_digest(&self) -> &str {
        &self.route_parity_digest
    }

    pub fn concrete_surface_width(&self) -> usize {
        self.evidence_source_count(
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
        )
    }

    pub fn synthetic_surface_width(&self) -> usize {
        self.evidence_source_count(
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
        )
    }

    pub fn concrete_surface_digest(&self) -> String {
        self.evidence_source_digest(
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
        )
    }

    pub fn synthetic_surface_digest(&self) -> String {
        self.evidence_source_digest(
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
        )
    }

    pub(crate) fn synthetic_surface_seams(&self) -> Vec<&'static str> {
        self.evidence_sources
            .iter()
            .filter_map(|(seam_key, source)| {
                (*source
                    == WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized)
                    .then_some(*seam_key)
            })
            .collect()
    }

    fn evidence_source_digest(
        &self,
        source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
    ) -> String {
        hash_parts(
            &self
                .evidence_sources
                .iter()
                .filter_map(|(seam_key, row_source)| {
                    (*row_source == source).then_some((*seam_key).to_string())
                })
                .collect::<Vec<_>>(),
        )
    }

    fn evidence_source_count(
        &self,
        source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
    ) -> usize {
        self.evidence_sources
            .values()
            .filter(|row_source| **row_source == source)
            .count()
    }

    pub(crate) fn evidence_source_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<WorthQueryLowerRuntimeRepresentativeEvidenceSource> {
        self.evidence_sources.get(seam_key.as_str()).copied()
    }

    #[cfg(test)]
    pub(super) fn with_evidence_source_override(
        mut self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
        source: WorthQueryLowerRuntimeRepresentativeEvidenceSource,
    ) -> Self {
        self.evidence_sources.insert(seam_key.as_str(), source);
        self
    }

    #[cfg(test)]
    pub(super) fn request_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<&WorthQueryLowerRuntimeCapabilityRequest> {
        self.requests_by_seam.get(seam_key.as_str())
    }

    #[cfg(test)]
    pub(super) fn route_plan_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<&WorthQueryLowerRuntimeRoutePlan> {
        self.route_plans_by_seam.get(seam_key.as_str())
    }

    #[cfg(test)]
    pub(super) fn boundary_receipt_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<&WorthQueryLowerRuntimeBoundaryExecutionReceipt> {
        self.receipts_by_seam.get(seam_key.as_str())
    }

    #[cfg(test)]
    pub(super) fn envelope_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<&WorthQueryLowerRuntimeBoundaryEnvelope> {
        self.envelopes_by_seam.get(seam_key.as_str())
    }
}

pub fn worth_query_lower_runtime_representative_surface(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryLowerRuntimeRepresentativeSurface {
    let concrete_rows = vec![
        representative_compose_read_row(),
        representative_execute_read_family_row(),
        representative_execute_read_family_in_basis_context_row(),
        representative_runtime_current_read_graph_row(),
        representative_runtime_basis_context_read_graph_row(),
        representative_live_view_schema_row(),
        representative_live_view_source_row(),
        representative_public_live_view_declaration_row(),
        representative_runtime_live_installation_orchestration_row(),
        representative_subscription_activation_row(),
        representative_subscription_continuity_row(resource_request),
        representative_preview_basis_row(),
        representative_basis_truth_view_readmission_row(resource_request),
        representative_basis_subscription_readmission_row(resource_request),
        representative_historical_bridge_lowering_row(resource_request),
        representative_effect_relational_mutation_row(),
        representative_effect_relational_merge_row(),
        representative_effect_bridge_writeback_row(),
        representative_write_authority_row(),
        representative_signal_invalidation_row(),
        representative_runtime_intent_authority_row(),
        representative_intent_runtime_execution_row(),
        representative_projection_query_receipts_row(),
        representative_projection_relational_row(),
        representative_projection_bridge_row(resource_request),
        representative_causal_bridge_materialization_row(),
        representative_frontier_evidence_row(),
    ];
    let concrete_seams = concrete_rows
        .iter()
        .map(|row| row.seam_key.as_str())
        .collect::<Vec<_>>();
    let mut rows = concrete_rows;

    for row in worth_query_lower_runtime_crossing_inventory().rows() {
        if concrete_seams.contains(&row.seam_key().as_str()) {
            continue;
        }
        rows.push(synthetic_inventory_row(row));
    }

    let requests = rows
        .iter()
        .map(|row| row.request.clone())
        .collect::<Vec<_>>();
    let eligibilities = rows
        .iter()
        .map(|row| row.eligibility.clone())
        .collect::<Vec<_>>();
    let route_plans = rows
        .iter()
        .filter_map(|row| row.route_plan.clone())
        .collect::<Vec<_>>();
    let boundary_receipts = rows
        .iter()
        .map(|row| row.boundary_receipt.clone())
        .collect::<Vec<_>>();
    let envelopes = rows
        .iter()
        .map(|row| row.envelope.clone())
        .collect::<Vec<_>>();
    let query_digest = digest_query_subjects(&requests);
    let route_parity_digest = hash_parts(&[
        normalized_parity_digest("compose-read", &envelopes),
        normalized_parity_digest("basis-readmission", &envelopes),
        hostile_parity_divergence_digest(&envelopes),
    ]);

    WorthQueryLowerRuntimeRepresentativeSurface {
        requests,
        eligibilities,
        route_plans,
        boundary_receipts,
        envelopes,
        query_digest,
        route_parity_digest,
        evidence_sources: collect_surface_map(&rows, |row| row.evidence_source),
        requests_by_seam: collect_surface_map(&rows, |row| row.request.clone()),
        route_plans_by_seam: rows
            .iter()
            .filter_map(|row| {
                row.route_plan
                    .clone()
                    .map(|plan| (row.seam_key.as_str(), plan))
            })
            .collect(),
        receipts_by_seam: collect_surface_map(&rows, |row| row.boundary_receipt.clone()),
        envelopes_by_seam: collect_surface_map(&rows, |row| row.envelope.clone()),
    }
}

fn digest_query_subjects(requests: &[WorthQueryLowerRuntimeCapabilityRequest]) -> String {
    hash_parts(
        &requests
            .iter()
            .map(|request| request.subject_digest().to_string())
            .collect::<Vec<_>>(),
    )
}

fn collect_surface_map<T: Clone>(
    rows: &[RepresentativeArtifacts],
    project: impl Fn(&RepresentativeArtifacts) -> T,
) -> BTreeMap<&'static str, T> {
    rows.iter()
        .map(|row| (row.seam_key.as_str(), project(row)))
        .collect()
}
