pub(crate) mod aspect_delta_failure_fields;
pub(crate) mod commit_conflict;
pub(crate) mod commit_preparation;
mod commit_result;
pub(crate) mod commit_semantic_result;
pub(crate) mod commit_summaries;
pub use commit_semantic_result::{
    RecoveredRelationalSemanticResult, RelationalCommitPublicationMeaning,
    RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal,
    RelationalCommitStructuralMeaning, RelationalCreatedEndpoint,
    RelationalCreatedEntityCorrespondence, RelationalCreatedEntityKey,
    RelationalCreatedRelationCorrespondence, RelationalCreatedRelationKey,
    RelationalInvariantCheckMeaning, RelationalInvariantCustomProvenanceMeaning,
    RelationalInvariantDecisionMeaning, RelationalInvariantExecutionMeaning,
    RelationalInvariantMetadataMeaning, RelationalInvariantProofScopeMeaning,
};
pub(crate) mod commit_validation;
pub(crate) mod conflict_class;
pub(crate) mod created_entity_bindings;
pub(crate) mod created_relation_bindings;
pub(crate) mod plan_artifacts;
pub(crate) mod rollback;
pub(crate) mod transaction_commit_error;

pub use aspect_delta_failure_fields::{
    AspectDeltaFailureFields, AspectDeltaPatchConstructionDenial, AspectDeltaPatchValueDenial,
    AspectDeltaRecordClass,
};
pub use commit_conflict::CommitConflict;
pub use commit_preparation::{
    CommitPreparationError, CommitPreparationReason, SelectedBranchRootDenialReason,
};
pub use commit_result::{
    CommitExecution, CommitOutcome, CommitPhaseTiming, CommitPublication, CommitResult,
    CommitSchemaSummary, CommitStructuralSummary,
};
pub use commit_validation::CommitValidationSummary;
mod live_commit_validation;
pub use conflict_class::{
    AspectFieldTargetRejectionReason, BulkImportRowDomain, BulkImportStage,
    BulkMutationAdmissionDenial, ConflictClass, EntityAuthoritativeAspectStateDenial,
    EntityCascadeDeleteMissingState, EntityFieldUpdateMissingState,
    MutationStateInconsistencyEvidence, RecordAllocationDenial, RecordAspectPatchDenial,
    RecordAspectPatchTarget, RelationEndpointUpdateMissingState,
};
pub(crate) use created_entity_bindings::CommitCreatedEntityBindings;
pub(crate) use created_relation_bindings::{
    CommitCreatedRelationBindings, CoupledOriginalRelationCreates,
};
pub use live_commit_validation::CommitValidation;
pub(crate) use plan_artifacts::merge_commit_mutation_plan_token;
pub use plan_artifacts::{
    AuthoritativeApplyPlan, MergeCommitMutationPlan, MergeExecutionStructuralSummary,
    MergeExecutionSummary, MergedCommitPlan, PublishedMergeExecutionAuthority, UndoRecord,
};
pub use rollback::{RollbackEffect, RollbackOutcome, RollbackSummary};
pub use transaction_commit_error::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
};

pub(crate) use commit_semantic_result::{
    ValidationLeafMemory, ValidationReadmissionMemory, ValidationTouchedMemory,
};

mod merge_execution_outcome;
pub use merge_execution_outcome::MergeExecutionOutcome;
