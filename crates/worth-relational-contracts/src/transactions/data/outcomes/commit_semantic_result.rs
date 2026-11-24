//! Original commit meaning shared by live assembly and canonical recovery.
use serde::{Deserialize, Serialize};

use crate::history::data::commit_receipt::RelationalCommitReceipt;
use crate::identity::data::PartitionId;
use crate::schema::data::DescriptorSemanticsVersion;
use crate::transactions::data::commit_log::{
    CommitAspectSummary, CommitChangeSummary, CommitHistorySummary,
};
use crate::transactions::data::mutation_planning::CommitTopology;
use crate::transactions::data::outcomes::commit_summaries::CommitSchemaSummary;
use crate::transactions::data::outcomes::commit_validation::CommitValidationSummary;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::data::groups::InvariantGroupSet;

pub mod creation_correspondence;
pub use creation_correspondence::{
    endpoint as canonical_created_endpoint, entity_key as canonical_created_entity_key,
};
pub mod validation;
pub use creation_correspondence::{
    RelationalCreatedEndpoint, RelationalCreatedEntityCorrespondence, RelationalCreatedEntityKey,
    RelationalCreatedRelationCorrespondence, RelationalCreatedRelationKey,
};
pub use validation::{
    RelationalInvariantCheckMeaning, RelationalInvariantCustomProvenanceMeaning,
    RelationalInvariantDecisionMeaning, RelationalInvariantExecutionMeaning,
    RelationalInvariantMetadataMeaning, RelationalInvariantProofScopeMeaning,
};
pub use validation::{ValidationLeafMemory, ValidationReadmissionMemory, ValidationTouchedMemory};

/// Immutable semantics produced once before publication. This is data, not a
/// claim that publication or settlement occurred. Live-only observations have
/// no fields here and cannot be invented by recovery.
///
/// ```compile_fail,E0609
/// # use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalCommitSemanticResultRefusal {
    MissingClientKeyText,
    ConflictingCreateCorrespondence,
    CreateCardinalityOverflow,
    RelationBirthMismatch {
        assigned: crate::identity::data::RelationId,
    },
    DuplicateAssignedCreation {
        record: crate::transactions::data::primitives::ExistingRecordTarget,
    },
    CreatedPatchMissing {
        record: crate::transactions::data::primitives::ExistingRecordTarget,
    },
    OriginalEntityKeyMissing {
        intent_ordinal: usize,
        entity_ordinal: usize,
    },
    OriginalRelationKeyMissing {
        intent_ordinal: usize,
        relation_ordinal: usize,
    },
    BulkRelationArityMismatch {
        intent_ordinal: usize,
    },
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

pub trait RelationalCommitSemanticResultFieldsPort {
    fn owner_structural(&self) -> &RelationalCommitStructuralMeaning;
    fn owner_receipt(&self) -> &RelationalCommitReceipt;
    fn owner_change(&self) -> &CommitChangeSummary;
    fn owner_changed_records(&self) -> &Vec<RecordRef>;
    fn owner_created_relations(&self) -> &Vec<RelationalCreatedRelationCorrespondence>;
    fn owner_validation(&self) -> &Vec<RelationalInvariantExecutionMeaning>;
    fn owner_decoded_memory_mut(
        &mut self,
    ) -> &mut Option<worth_execution::ExecutionMemoryReservation>;
    fn owner_publication(&self) -> &RelationalCommitPublicationMeaning;
    fn owner_aspects(&self) -> &CommitAspectSummary;
    fn owner_history(&self) -> &CommitHistorySummary;
    fn owner_schema(&self) -> &CommitSchemaSummary;
    fn owner_created_entities(&self) -> &Vec<RelationalCreatedEntityCorrespondence>;
}

impl RelationalCommitSemanticResultFieldsPort for RelationalCommitSemanticResult {
    fn owner_structural(&self) -> &RelationalCommitStructuralMeaning {
        &self.structural
    }
    fn owner_receipt(&self) -> &RelationalCommitReceipt {
        &self.receipt
    }
    fn owner_change(&self) -> &CommitChangeSummary {
        &self.change
    }
    fn owner_changed_records(&self) -> &Vec<RecordRef> {
        &self.changed_records
    }
    fn owner_created_relations(&self) -> &Vec<RelationalCreatedRelationCorrespondence> {
        &self.created_relations
    }
    fn owner_validation(&self) -> &Vec<RelationalInvariantExecutionMeaning> {
        &self.validation
    }
    fn owner_decoded_memory_mut(
        &mut self,
    ) -> &mut Option<worth_execution::ExecutionMemoryReservation> {
        &mut self.decoded_memory
    }
    fn owner_publication(&self) -> &RelationalCommitPublicationMeaning {
        &self.publication
    }
    fn owner_aspects(&self) -> &CommitAspectSummary {
        &self.aspects
    }
    fn owner_history(&self) -> &CommitHistorySummary {
        &self.history
    }
    fn owner_schema(&self) -> &CommitSchemaSummary {
        &self.schema
    }
    fn owner_created_entities(&self) -> &Vec<RelationalCreatedEntityCorrespondence> {
        &self.created_entities
    }
}

pub struct RelationalCommitSemanticResultParts {
    pub decoded_memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub receipt: RelationalCommitReceipt,
    pub created_entities: Vec<RelationalCreatedEntityCorrespondence>,
    pub created_relations: Vec<RelationalCreatedRelationCorrespondence>,
    pub changed_records: Vec<RecordRef>,
    pub structural: RelationalCommitStructuralMeaning,
    pub schema: CommitSchemaSummary,
    pub history: CommitHistorySummary,
    pub change: CommitChangeSummary,
    pub aspects: CommitAspectSummary,
    pub publication: RelationalCommitPublicationMeaning,
    pub validation: Vec<RelationalInvariantExecutionMeaning>,
}

pub fn assemble_commit_semantic_result(
    parts: RelationalCommitSemanticResultParts,
) -> RelationalCommitSemanticResult {
    RelationalCommitSemanticResult {
        decoded_memory: parts.decoded_memory,
        receipt: parts.receipt,
        created_entities: parts.created_entities,
        created_relations: parts.created_relations,
        changed_records: parts.changed_records,
        structural: parts.structural,
        schema: parts.schema,
        history: parts.history,
        change: parts.change,
        aspects: parts.aspects,
        publication: parts.publication,
        validation: parts.validation,
    }
}

pub fn assemble_recovered_relational_semantic_result(
    semantic: std::sync::Arc<RelationalCommitSemanticResult>,
) -> RecoveredRelationalSemanticResult {
    RecoveredRelationalSemanticResult { semantic }
}
