mod aspect_delta_failure_fields;
mod commit_conflict;
mod commit_preparation;
mod commit_result;
mod commit_validation;
mod conflict_class;
mod created_entity_bindings;
mod created_relation_bindings;
mod plan_artifacts;
mod rollback;
mod transaction_commit_error;

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
pub use commit_validation::{CommitValidation, CommitValidationSummary};
#[cfg(test)]
pub(crate) use conflict_class::EntityUpdateMissingState;
pub use conflict_class::{
    AspectFieldTargetRejectionReason, BulkImportRowDomain, BulkImportStage,
    BulkMutationAdmissionDenial, ConflictClass, EntityAuthoritativeAspectStateDenial,
    EntityCascadeDeleteMissingState, EntityFieldUpdateMissingState,
    MutationStateInconsistencyEvidence, RecordAllocationDenial, RecordAspectPatchDenial,
    RecordAspectPatchTarget, RelationEndpointUpdateMissingState,
};
pub(crate) use created_entity_bindings::CommitCreatedEntityBindings;
pub(crate) use created_relation_bindings::CommitCreatedRelationBindings;
pub(crate) use plan_artifacts::merge_commit_mutation_plan_token;
pub use plan_artifacts::{
    AuthoritativeApplyPlan, MergeCommitMutationPlan, MergeExecutionOutcome,
    MergeExecutionStructuralSummary, MergeExecutionSummary, MergedCommitPlan,
    PublishedMergeExecutionAuthority, UndoRecord,
};
pub use rollback::{RollbackEffect, RollbackOutcome, RollbackSummary};
pub use transaction_commit_error::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
};
