use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryLane,
    graph_read_access::WorthQueryGraphReadPlanReview,
};
use worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    WorthQueryInstalledApplicationQueryIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::{identity::VersionId, indexes::DerivedIndexGenerationId};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationDisclosureDecisionFact,
    WorthQueryApplicationDisclosureReceipt, WorthQueryApplicationDisclosureReceiptPosture,
    WorthQueryApplicationQueryAccessReceipt, WorthQueryApplicationQueryBasisPosture,
    WorthQueryApplicationQueryConsistency, WorthQueryApplicationQueryFreshness,
    WorthQueryApplicationQueryOmissionPosture, WorthQueryApplicationQueryWorkEvidence,
    WorthQueryApplicationResultBufferEvidence,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphReadCompletion, WorthQueryGraphReadDependencyEvidence,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StableReceiptObservation {
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: CanonicalDigestId,
    graph_authority_present: bool,
    provider_present: bool,
    basis: StableBasisIdentity,
    basis_version: VersionId,
    basis_posture: WorthQueryApplicationQueryBasisPosture,
    lane: WorthQueryApplicationQueryLane,
    consistency: WorthQueryApplicationQueryConsistency,
    freshness: WorthQueryApplicationQueryFreshness,
    predicate_index_generation: Option<DerivedIndexGenerationId>,
    target_identity_index_generation: Option<DerivedIndexGenerationId>,
    ordered_index_generation: Option<DerivedIndexGenerationId>,
    read_completion: StableReadCompletionObservation,
    canonical_work: StableCanonicalWorkObservation,
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
    total_work_units: usize,
    omission_posture: WorthQueryApplicationQueryOmissionPosture,
    disclosure: StableDisclosureObservation,
    result_buffer: Option<WorthQueryApplicationResultBufferEvidence>,
    basis_released: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StableReadCompletionObservation {
    plan_present: bool,
    binding: StableSchemaBinding,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    basis: StableBasisIdentity,
    basis_released: bool,
    review: WorthQueryGraphReadPlanReview,
    dependencies: WorthQueryGraphReadDependencyEvidence,
    resource_plan_present: bool,
    release_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableSchemaBinding {
    runtime_present: bool,
    generation: u64,
    package: CanonicalDigestId,
    schema: CanonicalDigestId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableBasisIdentity {
    runtime_instance_present: bool,
    branch: String,
    snapshot: u64,
    root_identity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableDisclosureObservation {
    posture: WorthQueryApplicationDisclosureReceiptPosture,
    classification: Option<String>,
    decisions: Vec<WorthQueryApplicationDisclosureDecisionFact>,
    disclosed: Vec<AspectValue>,
    omitted: Vec<AspectValue>,
    capability_authority_present: bool,
    decision_identity_present: bool,
    authorization_decision_fact_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableCanonicalWorkObservation {
    installation: StableInstallationCanonicalWork,
    admission: WorthQueryCanonicalWorkEvidence,
    execution: WorthQueryCanonicalWorkEvidence,
    provider_commit: WorthQueryCanonicalWorkEvidence,
    projection: WorthQueryCanonicalWorkEvidence,
    live_delivery: WorthQueryCanonicalWorkEvidence,
    retry_resolution: WorthQueryCanonicalWorkEvidence,
    recovery_inspection: WorthQueryCanonicalWorkEvidence,
    publication: WorthQueryCanonicalWorkEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableInstallationCanonicalWork {
    basis_preparations: u32,
    digest_derivations: u32,
    canonical_entries: u32,
    encoded_material_present: bool,
    prepared_only_encoded_bytes: usize,
    sha256_compression_blocks: usize,
    digest_text_materializations: u32,
}

impl StableReceiptObservation {
    pub(super) fn capture(receipt: &WorthQueryApplicationQueryAccessReceipt) -> Self {
        assert!(!receipt.graph_authority_identity().is_empty());
        assert!(!receipt.provider_identity().is_empty());
        let completion = receipt.read_completion();
        assert!(completion.session_identity().as_u64() > 0);
        assert!(completion.managed_run_identity().as_u64() > 0);
        assert_eq!(completion.basis_identity(), receipt.basis_identity());
        assert_eq!(
            completion.basis_release().identity(),
            receipt.basis_identity()
        );
        Self {
            query: receipt.query_identity().clone(),
            parameters: *receipt.parameter_binding_identity(),
            graph_authority_present: true,
            provider_present: true,
            basis: StableBasisIdentity::capture(receipt.basis_identity()),
            basis_version: receipt.basis_version(),
            basis_posture: receipt.basis_posture(),
            lane: receipt.lane(),
            consistency: receipt.consistency(),
            freshness: receipt.freshness(),
            predicate_index_generation: receipt.predicate_index_generation(),
            target_identity_index_generation: receipt.target_identity_index_generation(),
            ordered_index_generation: receipt.ordered_index_generation(),
            read_completion: StableReadCompletionObservation::capture(completion),
            canonical_work: StableCanonicalWorkObservation::capture(receipt.canonical_work()),
            authorization_work: receipt.authorization_work(),
            examined_candidate_count: receipt.examined_candidate_count(),
            projected_record_count: receipt.projected_record_count(),
            projected_field_count: receipt.projected_field_count(),
            adjacency_list_read_count: receipt.adjacency_list_read_count(),
            edge_scan_count: receipt.edge_scan_count(),
            ordering_comparison_count: receipt.ordering_comparison_count(),
            ordered_index_entry_count: receipt.ordered_index_entry_count(),
            target_identity_index_entry_count: receipt.target_identity_index_entry_count(),
            per_result_neighbor_lookup_count: receipt.per_result_neighbor_lookup_count(),
            fallback_count: receipt.fallback_count(),
            result_count: receipt.result_count(),
            truncation_count: receipt.truncation_count(),
            work: receipt.work(),
            total_work_units: receipt.total_work_units(),
            omission_posture: receipt.omission_posture(),
            disclosure: StableDisclosureObservation::capture(receipt.disclosure()),
            result_buffer: receipt.result_buffer(),
            basis_released: receipt.basis_released(),
        }
    }

    pub(super) fn assert_same(&self, other: &Self) {
        assert_eq!(self.query, other.query, "query identity");
        assert_eq!(self.parameters, other.parameters, "parameter identity");
        assert_eq!(self.graph_authority_present, other.graph_authority_present);
        assert_eq!(self.provider_present, other.provider_present);
        assert_eq!(self.basis, other.basis, "basis");
        assert_eq!(self.basis_version, other.basis_version, "basis version");
        assert_eq!(self.basis_posture, other.basis_posture, "basis posture");
        assert_eq!(self.lane, other.lane, "lane");
        assert_eq!(self.consistency, other.consistency, "consistency");
        assert_eq!(self.freshness, other.freshness, "freshness");
        assert_eq!(
            self.predicate_index_generation,
            other.predicate_index_generation
        );
        assert_eq!(
            self.target_identity_index_generation,
            other.target_identity_index_generation
        );
        assert_eq!(
            self.ordered_index_generation,
            other.ordered_index_generation
        );
        self.read_completion.assert_same(&other.read_completion);
        self.assert_same_work_and_result(other);
    }

    fn assert_same_work_and_result(&self, other: &Self) {
        assert_eq!(self.canonical_work, other.canonical_work, "canonical work");
        assert_eq!(self.authorization_work, other.authorization_work);
        assert_eq!(
            self.examined_candidate_count,
            other.examined_candidate_count
        );
        assert_eq!(self.projected_record_count, other.projected_record_count);
        assert_eq!(self.projected_field_count, other.projected_field_count);
        assert_eq!(
            self.adjacency_list_read_count,
            other.adjacency_list_read_count
        );
        assert_eq!(self.edge_scan_count, other.edge_scan_count);
        assert_eq!(
            self.ordering_comparison_count,
            other.ordering_comparison_count
        );
        assert_eq!(
            self.ordered_index_entry_count,
            other.ordered_index_entry_count
        );
        assert_eq!(
            self.target_identity_index_entry_count,
            other.target_identity_index_entry_count
        );
        assert_eq!(
            self.per_result_neighbor_lookup_count,
            other.per_result_neighbor_lookup_count
        );
        assert_eq!(self.fallback_count, other.fallback_count);
        assert_eq!(self.result_count, other.result_count);
        assert_eq!(self.truncation_count, other.truncation_count);
        assert_eq!(self.work, other.work, "query work");
        assert_eq!(self.total_work_units, other.total_work_units);
        assert_eq!(self.omission_posture, other.omission_posture);
        assert_eq!(self.disclosure, other.disclosure, "disclosure");
        assert_eq!(self.result_buffer, other.result_buffer, "result buffer");
        assert_eq!(self.basis_released, other.basis_released);
    }
}

impl StableReadCompletionObservation {
    pub(super) fn capture(completion: &WorthQueryGraphReadCompletion) -> Self {
        assert!(completion.session_identity().as_u64() > 0);
        assert!(completion.managed_run_identity().as_u64() > 0);
        assert!(completion.plan_identity().as_u64() > 0);
        assert!(!completion.release().resource_plan_identity().is_empty());
        Self {
            plan_present: true,
            binding: StableSchemaBinding::capture(completion.binding_identity()),
            obligation: completion.obligation_identity().clone(),
            basis: StableBasisIdentity::capture(completion.basis_identity()),
            basis_released: completion.basis_release().released(),
            review: completion.review().clone(),
            dependencies: completion.dependencies().clone(),
            resource_plan_present: true,
            release_scope: completion.release().scope(),
            released_reservation_count: completion.release().released_reservation_count(),
        }
    }

    pub(super) fn assert_same(&self, other: &Self) {
        assert_eq!(self.plan_present, other.plan_present);
        assert_eq!(self.binding, other.binding, "schema binding");
        assert_eq!(self.obligation, other.obligation, "graph obligation");
        assert_eq!(self.basis, other.basis, "read basis");
        assert_eq!(self.basis_released, other.basis_released);
        assert_eq!(self.review, other.review, "read review");
        assert_eq!(self.dependencies, other.dependencies, "read dependencies");
        assert_eq!(self.resource_plan_present, other.resource_plan_present);
        assert_eq!(self.release_scope, other.release_scope);
        assert_eq!(
            self.released_reservation_count,
            other.released_reservation_count
        );
    }
}

impl StableSchemaBinding {
    fn capture(
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> Self {
        assert!(binding.runtime_ordinal() > 0);
        Self {
            runtime_present: true,
            generation: binding.generation(),
            package: *binding.package_identity(),
            schema: *binding.schema_identity(),
        }
    }
}

impl StableBasisIdentity {
    fn capture(
        identity: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    ) -> Self {
        assert!(identity.runtime_instance_id() > 0);
        Self {
            runtime_instance_present: true,
            branch: identity.branch_id().0.clone(),
            snapshot: identity.snapshot_id().0,
            root_identity: identity.descriptor().root_identity(),
        }
    }
}

impl StableDisclosureObservation {
    fn capture(receipt: &WorthQueryApplicationDisclosureReceipt) -> Self {
        Self {
            posture: receipt.posture(),
            classification: receipt.classification().map(str::to_owned),
            decisions: receipt.decisions().to_vec(),
            disclosed: receipt.disclosed().to_vec(),
            omitted: receipt.omitted().to_vec(),
            capability_authority_present: receipt.capability_authority_identity().is_some(),
            decision_identity_present: receipt.decision_identity().is_some(),
            authorization_decision_fact_count: receipt.authorization_decision_fact_count(),
        }
    }
}

impl StableCanonicalWorkObservation {
    fn capture(work: WorthQueryCanonicalWorkPhases) -> Self {
        Self {
            installation: StableInstallationCanonicalWork::capture(work.installation()),
            admission: work.admission(),
            execution: work.execution(),
            provider_commit: work.provider_commit(),
            projection: work.projection(),
            live_delivery: work.live_delivery(),
            retry_resolution: work.retry_resolution(),
            recovery_inspection: work.recovery_inspection(),
            publication: work.publication(),
        }
    }
}

impl StableInstallationCanonicalWork {
    fn capture(work: WorthQueryCanonicalWorkEvidence) -> Self {
        assert!(
            work.canonical_material_allocation_bytes() >= work.canonical_encoded_bytes(),
            "amortized canonical storage must hold every encoded byte"
        );
        let prepared_only_encoded_bytes = work
            .canonical_encoded_bytes()
            .checked_sub(work.sha256_input_bytes())
            .expect("installation cannot hash more bytes than it canonically encodes");
        Self {
            basis_preparations: work.basis_preparations(),
            digest_derivations: work.digest_derivations(),
            canonical_entries: work.canonical_entries(),
            // Separate installations carry process-local ordinal encodings whose decimal width
            // is unrelated to the protected application value under comparison.
            encoded_material_present: work.canonical_encoded_bytes() > 0,
            // The installed native-contract catalog deliberately retains prepared canonical
            // bases without deriving substitute digest authority from them.
            prepared_only_encoded_bytes,
            sha256_compression_blocks: work.sha256_compression_blocks(),
            digest_text_materializations: work.digest_text_materializations(),
        }
    }
}
