//! Original commit meaning shared by live assembly and canonical recovery.
use serde::{Deserialize, Serialize};

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::PartitionId;
use crate::schema::data::DescriptorSemanticsVersion;
use crate::transactions::data::{
    CommitAspectSummary, CommitChangeSummary, CommitHistorySummary, CommitSchemaSummary,
    CommitTopology, CommitValidationSummary, RecordRef,
};
use crate::validation::data::InvariantGroupSet;

mod creation_correspondence;
pub(crate) use creation_correspondence::{
    endpoint as canonical_created_endpoint, entity_key as canonical_created_entity_key,
};
mod readmission;
mod validation;
pub use creation_correspondence::{
    RelationalCreatedEndpoint, RelationalCreatedEntityCorrespondence, RelationalCreatedEntityKey,
    RelationalCreatedRelationCorrespondence, RelationalCreatedRelationKey,
};
pub use validation::{
    RelationalInvariantCheckMeaning, RelationalInvariantCustomProvenanceMeaning,
    RelationalInvariantDecisionMeaning, RelationalInvariantExecutionMeaning,
    RelationalInvariantMetadataMeaning, RelationalInvariantProofScopeMeaning,
    RelationalValidationPreparationFailure,
};
pub(crate) use validation::{
    ValidationLeafMemory, ValidationReadmissionMemory, ValidationTouchedMemory,
};

/// Immutable semantics produced once before publication. This is data, not a
/// claim that publication or settlement occurred. Live-only observations have
/// no fields here and cannot be invented by recovery.
///
/// ```compile_fail,E0609
/// use worth_relational::facade::transactions::RelationalCommitSemanticResult;
/// fn no_live_snapshot(result: &RelationalCommitSemanticResult) {
///     let _ = &result.snapshot;
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCommitSemanticResult {
    pub(crate) receipt: RelationalCommitReceipt,
    pub(crate) created_entities: Vec<RelationalCreatedEntityCorrespondence>,
    pub(crate) created_relations: Vec<RelationalCreatedRelationCorrespondence>,
    pub(crate) changed_records: Vec<RecordRef>,
    pub(crate) structural: RelationalCommitStructuralMeaning,
    pub(crate) schema: CommitSchemaSummary,
    pub(crate) history: CommitHistorySummary,
    pub(crate) change: CommitChangeSummary,
    pub(crate) aspects: CommitAspectSummary,
    pub(crate) publication: RelationalCommitPublicationMeaning,
    pub(crate) validation: Vec<RelationalInvariantExecutionMeaning>,
    // Last: the semantic Arc keeps its own nested allocation admission.
    #[serde(skip)]
    pub(crate) decoded_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl PartialEq for RelationalCommitSemanticResult {
    fn eq(&self, other: &Self) -> bool {
        self.receipt == other.receipt
            && self.created_entities == other.created_entities
            && self.created_relations == other.created_relations
            && self.changed_records == other.changed_records
            && self.structural == other.structural
            && self.schema == other.schema
            && self.history == other.history
            && self.change == other.change
            && self.aspects == other.aspects
            && self.publication == other.publication
            && self.validation == other.validation
    }
}
impl Eq for RelationalCommitSemanticResult {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCommitStructuralMeaning {
    pub invariant_groups: InvariantGroupSet,
    pub commit_topology: CommitTopology,
    pub touched_partitions: Vec<PartitionId>,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCommitPublicationMeaning {
    pub patch_record_count: usize,
    pub lineage_event_count: usize,
    pub merge_parent_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCommitSemanticResultRefusal {
    MissingClientKeyText,
    ConflictingCreateCorrespondence,
    CreatePartitionMismatch,
    CreatedEndpointMissing,
    NonCanonicalCorrespondence,
    ReceiptMismatch,
    SchemaMismatch,
    ChangedRecordsMismatch,
    PublicationMismatch,
    ValidationMismatch,
    MissingOriginalSummary,
}

impl RelationalCommitSemanticResult {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        &self.receipt
    }
    pub fn created_entities(&self) -> &[RelationalCreatedEntityCorrespondence] {
        &self.created_entities
    }
    pub fn created_relations(&self) -> &[RelationalCreatedRelationCorrespondence] {
        &self.created_relations
    }
    pub fn changed_records(&self) -> &[RecordRef] {
        &self.changed_records
    }
    pub fn structural(&self) -> &RelationalCommitStructuralMeaning {
        &self.structural
    }
    pub fn schema(&self) -> &CommitSchemaSummary {
        &self.schema
    }
    pub fn history(&self) -> &CommitHistorySummary {
        &self.history
    }
    pub fn change(&self) -> &CommitChangeSummary {
        &self.change
    }
    pub fn aspects(&self) -> &CommitAspectSummary {
        &self.aspects
    }
    pub fn publication(&self) -> &RelationalCommitPublicationMeaning {
        &self.publication
    }
    pub fn validation(&self) -> &[RelationalInvariantExecutionMeaning] {
        &self.validation
    }
    pub fn validation_summary(&self) -> CommitValidationSummary {
        validation::summarize(&self.validation)
    }
}

/// A read-only view of original semantic data obtained through Native source
/// admission. It carries no live snapshot, timing or result seal.
#[derive(Clone, Debug)]
pub struct RecoveredRelationalSemanticResult {
    pub(crate) semantic: std::sync::Arc<RelationalCommitSemanticResult>,
}

impl RecoveredRelationalSemanticResult {
    pub fn semantic(&self) -> &RelationalCommitSemanticResult {
        &self.semantic
    }
}
