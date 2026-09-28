use super::WorthQueryApplicationBasisIdentity;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkPhases, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::indexes::DerivedIndexGenerationId;

use super::read_execution::NonLiveKernelReceiptEvidence;
use super::WorthQueryApplicationResultBufferEvidence;
use super::{
    disclosure::WorthQueryApplicationDisclosureReceipt,
    WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationQueryBasisPosture,
    WorthQueryApplicationQueryConsistency, WorthQueryApplicationQueryFreshness,
};

mod omission_posture;
mod work_evidence;

pub use omission_posture::WorthQueryApplicationQueryOmissionPosture;
pub use work_evidence::WorthQueryApplicationQueryWorkEvidence;

/// Record of how one application query read was served: which query and
/// parameters, on which basis, under which consistency and freshness, what it
/// disclosed, and how much work it did.
///
/// Every query result, continuation page, and live update carries one through
/// `receipt()`. It is descriptive evidence for audit and budgeting, not
/// authority to read again.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryAccessReceipt {
    query_identity: WorthQueryInstalledApplicationQueryIdentity,
    parameter_binding_identity: CanonicalDigestId,
    graph_authority_identity: String,
    provider_identity: String,
    basis_identity: WorthQueryApplicationBasisIdentity,
    basis_version: VersionId,
    basis_posture: WorthQueryApplicationQueryBasisPosture,
    lane: WorthQueryApplicationQueryLane,
    consistency: WorthQueryApplicationQueryConsistency,
    freshness: WorthQueryApplicationQueryFreshness,
    predicate_index_generation: Option<DerivedIndexGenerationId>,
    target_identity_index_generation: Option<DerivedIndexGenerationId>,
    ordered_index_generation: Option<DerivedIndexGenerationId>,
    read_completion: crate::domain_computation::provider_session::WorthQueryGraphReadCompletion,
    canonical_work: WorthQueryCanonicalWorkPhases,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    examined_candidate_count: usize,
    projected_record_count: usize,
    projected_field_count: usize,
    adjacency_list_read_count: usize,
    edge_scan_count: usize,
    ordering_comparison_count: usize,
    ordered_index_entry_count: usize,
    target_identity_index_entry_count: usize,
    per_result_neighbor_lookup_count: usize,
    fallback_count: usize,
    result_count: usize,
    truncation_count: usize,
    work: WorthQueryApplicationQueryWorkEvidence,
    omission_posture: WorthQueryApplicationQueryOmissionPosture,
    disclosure: WorthQueryApplicationDisclosureReceipt,
    result_buffer: Option<WorthQueryApplicationResultBufferEvidence>,
    basis_released: bool,
}

pub(super) struct WorthQueryApplicationQueryAccessReceiptParts {
    pub query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub parameter_binding_identity: CanonicalDigestId,
    pub graph_authority_identity: String,
    pub provider_identity: String,
    pub basis_identity: WorthQueryApplicationBasisIdentity,
    pub basis_version: VersionId,
    pub basis_posture: WorthQueryApplicationQueryBasisPosture,
    pub lane: WorthQueryApplicationQueryLane,
    pub consistency: WorthQueryApplicationQueryConsistency,
    pub freshness: WorthQueryApplicationQueryFreshness,
    pub predicate_index_generation: Option<DerivedIndexGenerationId>,
    pub target_identity_index_generation: Option<DerivedIndexGenerationId>,
    pub ordered_index_generation: Option<DerivedIndexGenerationId>,
    pub read_completion: crate::domain_computation::provider_session::WorthQueryGraphReadCompletion,
    pub canonical_work: WorthQueryCanonicalWorkPhases,
    pub authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    pub examined_candidate_count: usize,
    pub predicate_work_units: usize,
    pub projected_record_count: usize,
    pub projected_field_count: usize,
    pub adjacency_list_read_count: usize,
    pub edge_scan_count: usize,
    pub ordering_comparison_count: usize,
    pub ordered_index_entry_count: usize,
    pub target_identity_index_entry_count: usize,
    pub per_result_neighbor_lookup_count: usize,
    pub fallback_count: usize,
    pub result_count: usize,
    pub truncation_count: usize,
    pub total_work_units: usize,
    pub result_buffer: Option<WorthQueryApplicationResultBufferEvidence>,
    pub basis_released: bool,
    pub disclosure: WorthQueryApplicationDisclosureReceipt,
}

pub(super) struct WorthQueryApplicationQueryReceiptIdentity {
    pub query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub parameter_binding_identity: CanonicalDigestId,
    pub graph_authority_identity: String,
    pub provider_identity: String,
}

pub(super) struct WorthQueryApplicationQueryReceiptBasis {
    pub identity: WorthQueryApplicationBasisIdentity,
    pub version: VersionId,
    pub posture: WorthQueryApplicationQueryBasisPosture,
    pub lane: WorthQueryApplicationQueryLane,
    pub consistency: WorthQueryApplicationQueryConsistency,
    pub freshness: WorthQueryApplicationQueryFreshness,
    pub released: bool,
}

impl WorthQueryApplicationQueryAccessReceipt {
    pub(super) fn from_non_live_kernel(
        identity: WorthQueryApplicationQueryReceiptIdentity,
        basis: WorthQueryApplicationQueryReceiptBasis,
        read_completion: crate::domain_computation::provider_session::WorthQueryGraphReadCompletion,
        canonical_work: WorthQueryCanonicalWorkPhases,
        authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
        disclosure: WorthQueryApplicationDisclosureReceipt,
        kernel: NonLiveKernelReceiptEvidence,
    ) -> Self {
        let raw = kernel.read;
        Self::new(WorthQueryApplicationQueryAccessReceiptParts {
            query_identity: identity.query_identity,
            parameter_binding_identity: identity.parameter_binding_identity,
            graph_authority_identity: identity.graph_authority_identity,
            provider_identity: identity.provider_identity,
            basis_identity: basis.identity,
            basis_version: basis.version,
            basis_posture: basis.posture,
            lane: basis.lane,
            consistency: basis.consistency,
            freshness: basis.freshness,
            predicate_index_generation: raw.predicate_index_generation,
            target_identity_index_generation: raw.target_identity_index_generation,
            ordered_index_generation: raw.ordered_index_generation,
            read_completion,
            canonical_work,
            authorization_work,
            examined_candidate_count: raw.examined_candidates,
            predicate_work_units: raw.predicate_work_units,
            projected_record_count: raw.projected_records,
            projected_field_count: raw.projected_fields,
            adjacency_list_read_count: raw.adjacency_lists_read,
            edge_scan_count: raw.relation_records_examined,
            ordering_comparison_count: raw.ordering_comparisons,
            ordered_index_entry_count: raw.ordered_index_entries_examined,
            target_identity_index_entry_count: raw.target_identity_index_entries_examined,
            per_result_neighbor_lookup_count: 0,
            fallback_count: 0,
            result_count: kernel.result_count,
            truncation_count: kernel.truncation_count,
            total_work_units: raw.actual_work,
            result_buffer: Some(kernel.result_buffer),
            basis_released: basis.released,
            disclosure,
        })
    }

    pub const fn query_identity(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query_identity
    }

    pub const fn parameter_binding_identity(&self) -> &CanonicalDigestId {
        &self.parameter_binding_identity
    }

    pub fn graph_authority_identity(&self) -> &str {
        &self.graph_authority_identity
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub const fn basis_identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis_identity
    }

    pub const fn basis_version(&self) -> VersionId {
        self.basis_version
    }

    pub const fn basis_posture(&self) -> WorthQueryApplicationQueryBasisPosture {
        self.basis_posture
    }

    pub const fn lane(&self) -> WorthQueryApplicationQueryLane {
        self.lane
    }

    pub const fn consistency(&self) -> WorthQueryApplicationQueryConsistency {
        self.consistency
    }

    pub const fn freshness(&self) -> WorthQueryApplicationQueryFreshness {
        self.freshness
    }

    pub const fn predicate_index_generation(&self) -> Option<DerivedIndexGenerationId> {
        self.predicate_index_generation
    }

    pub const fn ordered_index_generation(&self) -> Option<DerivedIndexGenerationId> {
        self.ordered_index_generation
    }

    pub const fn target_identity_index_generation(&self) -> Option<DerivedIndexGenerationId> {
        self.target_identity_index_generation
    }

    pub const fn graph_read_plan(
        &self,
    ) -> &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview {
        self.read_completion.review()
    }

    pub const fn read_completion(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryGraphReadCompletion {
        &self.read_completion
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    pub const fn authorization_work(&self) -> WorthQueryApplicationAuthorizationWorkEvidence {
        self.authorization_work
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.examined_candidate_count
    }

    pub const fn projected_record_count(&self) -> usize {
        self.projected_record_count
    }

    pub const fn projected_field_count(&self) -> usize {
        self.projected_field_count
    }

    pub const fn adjacency_list_read_count(&self) -> usize {
        self.adjacency_list_read_count
    }

    pub const fn edge_scan_count(&self) -> usize {
        self.edge_scan_count
    }

    pub const fn ordering_comparison_count(&self) -> usize {
        self.ordering_comparison_count
    }

    pub const fn ordered_index_entry_count(&self) -> usize {
        self.ordered_index_entry_count
    }

    pub const fn target_identity_index_entry_count(&self) -> usize {
        self.target_identity_index_entry_count
    }

    pub const fn per_result_neighbor_lookup_count(&self) -> usize {
        self.per_result_neighbor_lookup_count
    }

    pub const fn fallback_count(&self) -> usize {
        self.fallback_count
    }

    pub const fn result_count(&self) -> usize {
        self.result_count
    }

    pub const fn truncation_count(&self) -> usize {
        self.truncation_count
    }

    pub const fn work(&self) -> WorthQueryApplicationQueryWorkEvidence {
        self.work
    }

    pub const fn total_work_units(&self) -> usize {
        self.work
            .total_work_units()
            .saturating_add(self.authorization_work.observation_work_units())
    }

    pub const fn omission_posture(&self) -> WorthQueryApplicationQueryOmissionPosture {
        self.omission_posture
    }

    pub const fn disclosure(&self) -> &WorthQueryApplicationDisclosureReceipt {
        &self.disclosure
    }

    pub const fn result_buffer(&self) -> Option<WorthQueryApplicationResultBufferEvidence> {
        self.result_buffer
    }

    pub const fn basis_released(&self) -> bool {
        self.basis_released
    }

    pub(super) fn new(parts: WorthQueryApplicationQueryAccessReceiptParts) -> Self {
        Self {
            query_identity: parts.query_identity,
            parameter_binding_identity: parts.parameter_binding_identity,
            graph_authority_identity: parts.graph_authority_identity,
            provider_identity: parts.provider_identity,
            basis_identity: parts.basis_identity,
            basis_version: parts.basis_version,
            basis_posture: parts.basis_posture,
            lane: parts.lane,
            consistency: parts.consistency,
            freshness: parts.freshness,
            predicate_index_generation: parts.predicate_index_generation,
            target_identity_index_generation: parts.target_identity_index_generation,
            ordered_index_generation: parts.ordered_index_generation,
            read_completion: parts.read_completion,
            canonical_work: parts.canonical_work,
            authorization_work: parts.authorization_work,
            examined_candidate_count: parts.examined_candidate_count,
            projected_record_count: parts.projected_record_count,
            projected_field_count: parts.projected_field_count,
            adjacency_list_read_count: parts.adjacency_list_read_count,
            edge_scan_count: parts.edge_scan_count,
            ordering_comparison_count: parts.ordering_comparison_count,
            ordered_index_entry_count: parts.ordered_index_entry_count,
            target_identity_index_entry_count: parts.target_identity_index_entry_count,
            per_result_neighbor_lookup_count: parts.per_result_neighbor_lookup_count,
            fallback_count: parts.fallback_count,
            result_count: parts.result_count,
            truncation_count: parts.truncation_count,
            work: WorthQueryApplicationQueryWorkEvidence::new(
                parts.predicate_work_units,
                parts
                    .adjacency_list_read_count
                    .saturating_add(parts.edge_scan_count),
                parts.ordering_comparison_count,
                parts.ordered_index_entry_count,
                parts
                    .projected_record_count
                    .saturating_add(parts.projected_field_count),
                parts.total_work_units,
            ),
            omission_posture: if !parts.disclosure.has_omissions() {
                WorthQueryApplicationQueryOmissionPosture::NoOmission
            } else {
                WorthQueryApplicationQueryOmissionPosture::GovernedOmission
            },
            disclosure: parts.disclosure,
            result_buffer: parts.result_buffer,
            basis_released: parts.basis_released,
        }
    }
}
