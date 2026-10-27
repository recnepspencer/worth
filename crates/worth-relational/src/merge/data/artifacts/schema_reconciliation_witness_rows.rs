use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::identity::data::KindId;
use crate::merge::data::MergePolicyDecisionBoundary;
use crate::schema::data::{
    SchemaId, SchemaReconciliationClassification, SchemaReconciliationPolicy, SchemaVersionId,
};
use crate::transactions::data::RecordRef;

use super::super::{IdentityBasisKind, IdentityBasisScope};
use super::schema_reconciliation_witness_transition::{
    derive_schema_reconciliation_truth, DerivedSchemaReconciliationTruth,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalSchemaReconciliationWitnessPosture {
    Reconciled,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalSchemaReconciliationWitnessDenial {
    UnvalidatedSchemaCorrespondence,
    ManualResolutionRequired,
    PolicyRejected,
    StructuralIncompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalSchemaReconciliationBasisRow {
    pub source_kind_id: Option<KindId>,
    pub target_kind_id: Option<KindId>,
    pub source_schema_id: Option<SchemaId>,
    pub source_schema_version_id: Option<SchemaVersionId>,
    pub target_schema_id: Option<SchemaId>,
    pub target_schema_version_id: Option<SchemaVersionId>,
    pub registry_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalSchemaReconciliationCorrespondenceLinkRow {
    pub scope: IdentityBasisScope,
    pub basis: IdentityBasisKind,
    pub source_record: RecordRef,
    pub target_record: RecordRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalSchemaReconciliationWitnessRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
    basis: RelationalSchemaReconciliationBasisRow,
    source_only_aspect_count: usize,
    target_only_aspect_count: usize,
    divergent_aspect_count: usize,
    unavailable_aspect_count: usize,
    decision_boundary: MergePolicyDecisionBoundary,
    relation_endpoint_divergence: bool,
    correspondence_linkage: Option<RelationalSchemaReconciliationCorrespondenceLinkRow>,
    classification: SchemaReconciliationClassification,
    policy: Option<SchemaReconciliationPolicy>,
    denial: Option<RelationalSchemaReconciliationWitnessDenial>,
    posture: RelationalSchemaReconciliationWitnessPosture,
    row_digest: String,
}

pub struct RelationalSchemaReconciliationWitnessRowInput {
    pub record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub basis: RelationalSchemaReconciliationBasisRow,
    pub source_only_aspect_count: usize,
    pub target_only_aspect_count: usize,
    pub divergent_aspect_count: usize,
    pub unavailable_aspect_count: usize,
    pub decision_boundary: MergePolicyDecisionBoundary,
    pub relation_endpoint_divergence: bool,
    pub correspondence_linkage: Option<RelationalSchemaReconciliationCorrespondenceLinkRow>,
}

impl RelationalSchemaReconciliationWitnessRow {
    pub(crate) fn retained(input: RelationalSchemaReconciliationWitnessRowInput) -> Self {
        let derived = derive_schema_reconciliation_truth(&input);
        let row_digest = schema_reconciliation_row_digest(&input, &derived);
        Self {
            record: input.record,
            target_record: input.target_record,
            basis: input.basis,
            source_only_aspect_count: input.source_only_aspect_count,
            target_only_aspect_count: input.target_only_aspect_count,
            divergent_aspect_count: input.divergent_aspect_count,
            unavailable_aspect_count: input.unavailable_aspect_count,
            decision_boundary: input.decision_boundary,
            relation_endpoint_divergence: input.relation_endpoint_divergence,
            correspondence_linkage: input.correspondence_linkage,
            classification: derived.classification,
            policy: derived.policy,
            denial: derived.denial,
            posture: derived.posture,
            row_digest,
        }
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }

    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }

    pub fn basis(&self) -> &RelationalSchemaReconciliationBasisRow {
        &self.basis
    }

    pub fn correspondence_linkage(
        &self,
    ) -> Option<&RelationalSchemaReconciliationCorrespondenceLinkRow> {
        self.correspondence_linkage.as_ref()
    }

    pub fn source_only_aspect_count(&self) -> usize {
        self.source_only_aspect_count
    }

    pub fn target_only_aspect_count(&self) -> usize {
        self.target_only_aspect_count
    }

    pub fn divergent_aspect_count(&self) -> usize {
        self.divergent_aspect_count
    }

    pub fn unavailable_aspect_count(&self) -> usize {
        self.unavailable_aspect_count
    }

    pub fn decision_boundary(&self) -> MergePolicyDecisionBoundary {
        self.decision_boundary
    }

    pub fn relation_endpoint_divergence(&self) -> bool {
        self.relation_endpoint_divergence
    }

    pub fn classification(&self) -> SchemaReconciliationClassification {
        self.classification
    }

    pub fn policy(&self) -> Option<SchemaReconciliationPolicy> {
        self.policy
    }

    pub fn denial(&self) -> Option<RelationalSchemaReconciliationWitnessDenial> {
        self.denial
    }

    pub fn posture(&self) -> RelationalSchemaReconciliationWitnessPosture {
        self.posture
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }

    fn input(&self) -> RelationalSchemaReconciliationWitnessRowInput {
        RelationalSchemaReconciliationWitnessRowInput {
            record: self.record.clone(),
            target_record: self.target_record.clone(),
            basis: self.basis.clone(),
            source_only_aspect_count: self.source_only_aspect_count,
            target_only_aspect_count: self.target_only_aspect_count,
            divergent_aspect_count: self.divergent_aspect_count,
            unavailable_aspect_count: self.unavailable_aspect_count,
            decision_boundary: self.decision_boundary,
            relation_endpoint_divergence: self.relation_endpoint_divergence,
            correspondence_linkage: self.correspondence_linkage.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalSchemaReconciliationWitnessRowWire {
    record: RecordRef,
    target_record: Option<RecordRef>,
    basis: RelationalSchemaReconciliationBasisRow,
    source_only_aspect_count: usize,
    target_only_aspect_count: usize,
    divergent_aspect_count: usize,
    unavailable_aspect_count: usize,
    decision_boundary: MergePolicyDecisionBoundary,
    relation_endpoint_divergence: bool,
    correspondence_linkage: Option<RelationalSchemaReconciliationCorrespondenceLinkRow>,
    classification: SchemaReconciliationClassification,
    policy: Option<SchemaReconciliationPolicy>,
    denial: Option<RelationalSchemaReconciliationWitnessDenial>,
    posture: RelationalSchemaReconciliationWitnessPosture,
    row_digest: String,
}

impl<'de> Deserialize<'de> for RelationalSchemaReconciliationWitnessRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalSchemaReconciliationWitnessRowWire::deserialize(deserializer)?;
        let row = Self {
            record: wire.record,
            target_record: wire.target_record,
            basis: wire.basis,
            source_only_aspect_count: wire.source_only_aspect_count,
            target_only_aspect_count: wire.target_only_aspect_count,
            divergent_aspect_count: wire.divergent_aspect_count,
            unavailable_aspect_count: wire.unavailable_aspect_count,
            decision_boundary: wire.decision_boundary,
            relation_endpoint_divergence: wire.relation_endpoint_divergence,
            correspondence_linkage: wire.correspondence_linkage,
            classification: wire.classification,
            policy: wire.policy,
            denial: wire.denial,
            posture: wire.posture,
            row_digest: wire.row_digest,
        };
        let input = row.input();
        let derived = derive_schema_reconciliation_truth(&input);
        if row.classification != derived.classification
            || row.policy != derived.policy
            || row.denial != derived.denial
            || row.posture != derived.posture
        {
            return Err(D::Error::custom(
                "schema reconciliation witness row truth does not match retained schema decision inputs",
            ));
        }
        if row.row_digest != schema_reconciliation_row_digest(&input, &derived) {
            return Err(D::Error::custom(
                "schema reconciliation witness row digest does not match retained schema truth",
            ));
        }
        Ok(row)
    }
}

fn schema_reconciliation_row_digest(
    input: &RelationalSchemaReconciliationWitnessRowInput,
    derived: &DerivedSchemaReconciliationTruth,
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.schema_reconciliation_row.v1");
    bytes.extend_from_slice(
        &rmp_serde::to_vec_named(&(
            &input.record,
            &input.target_record,
            &input.basis,
            input.source_only_aspect_count,
            input.target_only_aspect_count,
            input.divergent_aspect_count,
            input.unavailable_aspect_count,
            input.decision_boundary,
            input.relation_endpoint_divergence,
            &input.correspondence_linkage,
            derived.classification,
            derived.policy,
            derived.denial,
            derived.posture,
        ))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "schema reconciliation witness row must encode"),
    );
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
