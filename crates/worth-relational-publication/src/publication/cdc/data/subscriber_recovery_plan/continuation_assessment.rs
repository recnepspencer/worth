use crate::publication::cdc::data::normalized_continuation_proof::NormalizedContinuationProof;
use contracts::publication::cdc::data::subscriber_contract::SubscriberContinuationSummary;
use contracts::schema::data::continuity::boundary_fingerprint::SchemaBoundaryFingerprint;
use contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;
use contracts::schema::data::continuity::transition_classification::SchemaContinuationClassification;
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberContinuationAssessment {
    crossed_boundaries: Vec<SchemaBoundaryFingerprint>,
    continuation_outcome: SchemaContinuationClassification,
    contract_upgrade_applied: bool,
    normalized_continuation_proof: NormalizedContinuationProof,
    continuation_summary: SubscriberContinuationSummary,
    boundary_assessments: Vec<SubscriberBoundaryAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberBoundaryAssessment {
    boundary_fingerprint: SchemaBoundaryFingerprint,
    descriptor_continuation: SchemaContinuationClassification,
    subscriber_outcome: SchemaContinuationClassification,
    changed_strata:
        Vec<contracts::schema::data::continuity::transition_classification::SchemaStratum>,
    contract_consumes_boundary: bool,
}

impl SubscriberContinuationAssessment {
    pub(crate) fn new(
        crossed_boundaries: Vec<SchemaBoundaryFingerprint>,
        continuation_outcome: SchemaContinuationClassification,
        contract_upgrade_applied: bool,
        normalized_continuation_proof: NormalizedContinuationProof,
        continuation_summary: SubscriberContinuationSummary,
        boundary_assessments: Vec<SubscriberBoundaryAssessment>,
    ) -> Self {
        Self {
            crossed_boundaries,
            continuation_outcome,
            contract_upgrade_applied,
            normalized_continuation_proof,
            continuation_summary,
            boundary_assessments,
        }
    }

    pub(crate) fn unchanged(
        contract_id: String,
        descriptor_semantics_version: DescriptorSemanticsVersion,
    ) -> Self {
        Self::new(
            Vec::new(),
            SchemaContinuationClassification::ContinueUnchanged,
            false,
            NormalizedContinuationProof::empty(descriptor_semantics_version),
            SubscriberContinuationSummary::unchanged(contract_id, descriptor_semantics_version),
            Vec::new(),
        )
    }

    pub fn crossed_boundaries(&self) -> &[SchemaBoundaryFingerprint] {
        &self.crossed_boundaries
    }

    pub fn continuation_outcome(&self) -> SchemaContinuationClassification {
        self.continuation_outcome
    }

    pub fn contract_upgrade_applied(&self) -> bool {
        self.contract_upgrade_applied
    }

    pub fn normalized_continuation_proof(&self) -> &NormalizedContinuationProof {
        &self.normalized_continuation_proof
    }

    pub fn continuation_summary(&self) -> &SubscriberContinuationSummary {
        &self.continuation_summary
    }

    pub fn boundary_assessments(&self) -> &[SubscriberBoundaryAssessment] {
        &self.boundary_assessments
    }
}

impl SubscriberBoundaryAssessment {
    pub(crate) fn new(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        descriptor_continuation: SchemaContinuationClassification,
        subscriber_outcome: SchemaContinuationClassification,
        changed_strata: Vec<
            contracts::schema::data::continuity::transition_classification::SchemaStratum,
        >,
        contract_consumes_boundary: bool,
    ) -> Self {
        Self {
            boundary_fingerprint,
            descriptor_continuation,
            subscriber_outcome,
            changed_strata,
            contract_consumes_boundary,
        }
    }

    pub fn boundary_fingerprint(&self) -> SchemaBoundaryFingerprint {
        self.boundary_fingerprint
    }

    pub fn descriptor_continuation(&self) -> SchemaContinuationClassification {
        self.descriptor_continuation
    }

    pub fn subscriber_outcome(&self) -> SchemaContinuationClassification {
        self.subscriber_outcome
    }

    pub fn changed_strata(
        &self,
    ) -> &[contracts::schema::data::continuity::transition_classification::SchemaStratum] {
        &self.changed_strata
    }

    pub fn contract_consumes_boundary(&self) -> bool {
        self.contract_consumes_boundary
    }
}

#[doc(hidden)]
pub trait SubscriberContinuationAssessmentOwnerPort: Sized {
    fn new(
        crossed_boundaries: Vec<SchemaBoundaryFingerprint>,
        continuation_outcome: SchemaContinuationClassification,
        contract_upgrade_applied: bool,
        normalized_continuation_proof: NormalizedContinuationProof,
        continuation_summary: SubscriberContinuationSummary,
        boundary_assessments: Vec<SubscriberBoundaryAssessment>,
    ) -> Self;

    fn unchanged(
        contract_id: String,
        descriptor_semantics_version: DescriptorSemanticsVersion,
    ) -> Self;
}

impl SubscriberContinuationAssessmentOwnerPort for SubscriberContinuationAssessment {
    fn new(
        crossed_boundaries: Vec<SchemaBoundaryFingerprint>,
        continuation_outcome: SchemaContinuationClassification,
        contract_upgrade_applied: bool,
        normalized_continuation_proof: NormalizedContinuationProof,
        continuation_summary: SubscriberContinuationSummary,
        boundary_assessments: Vec<SubscriberBoundaryAssessment>,
    ) -> Self {
        Self::new(
            crossed_boundaries,
            continuation_outcome,
            contract_upgrade_applied,
            normalized_continuation_proof,
            continuation_summary,
            boundary_assessments,
        )
    }

    fn unchanged(
        contract_id: String,
        descriptor_semantics_version: DescriptorSemanticsVersion,
    ) -> Self {
        Self::unchanged(contract_id, descriptor_semantics_version)
    }
}

#[doc(hidden)]
pub trait SubscriberBoundaryAssessmentOwnerPort: Sized {
    fn new(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        descriptor_continuation: SchemaContinuationClassification,
        subscriber_outcome: SchemaContinuationClassification,
        changed_strata: Vec<
            contracts::schema::data::continuity::transition_classification::SchemaStratum,
        >,
        contract_consumes_boundary: bool,
    ) -> Self;
}

impl SubscriberBoundaryAssessmentOwnerPort for SubscriberBoundaryAssessment {
    fn new(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        descriptor_continuation: SchemaContinuationClassification,
        subscriber_outcome: SchemaContinuationClassification,
        changed_strata: Vec<
            contracts::schema::data::continuity::transition_classification::SchemaStratum,
        >,
        contract_consumes_boundary: bool,
    ) -> Self {
        Self::new(
            boundary_fingerprint,
            descriptor_continuation,
            subscriber_outcome,
            changed_strata,
            contract_consumes_boundary,
        )
    }
}
