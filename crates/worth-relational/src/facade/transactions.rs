//! Transaction vocabulary exposed to Relational consumers.

pub use crate::execution::RelationalExecutionDenialCause;

pub use crate::transactions::data::{
    planned_aspect_field_locator, planned_single_field_locator, ApplyEntityAspectPatchIntent,
    ApplyRelationAspectPatchIntent, AspectEmissionTrace, AspectEvaluationTrace,
    AspectEvaluationTraceRow, AspectFieldPatch, AspectLifecycleTransitionClass,
    AspectTagAccuracyReport, AspectTraceEvidence, AuthoritativeApplyPlan, BulkEntityCreateIntent,
    BulkMutationLineagePlan, BulkMutationLocalityFootprint, BulkMutationNamingPlan,
    BulkMutationProvenancePlan, BulkMutationScope, BulkRelationCreateIntent, CommitAspectSummary,
    CommitChangeSummary, CommitConflict, CommitExecutionDenial, CommitExecutionDenialKind,
    CommitHistorySummary, CommitLog, CommitOutcome, CommitPatchBudgetSummary, CommitPhase,
    CommitPhaseTiming, CommitPreparationError, CommitPreparationReason, CommitPublicationSummary,
    CommitResult, CommitSchemaSummary, CommitStructuralSummary, CommitSummary, CommitTopology,
    CommitTraceEvent, ConflictClass, CreateIntent, CreatedEntityRef, CreatedRelationRef,
    CrossContextEndpointClass, DeleteEntityIntent, DeleteRelationIntent, EntityAspectCreateIntent,
    EntityMutationIntent, EntityReference, EntitySpec, LineageSafeBulkMutationBatch,
    MergeCommitMutationPlan, MergeExecutionOutcome, MergeExecutionStructuralSummary,
    MergeExecutionSummary, MergedCommitPlan, MutationIntent, NamingStableBulkMutationBatch,
    PatchVsTruthDeltaReport, PlannedBulkMutationBatch, PlannedLineageTransition,
    ProvenanceCompleteBulkMutationBatch, PublishedMergeExecutionAuthority, RecordRef,
    RelationAspectCreateIntent, RelationMutationIntent, RelationScope, RelationSpec,
    ReplaceEntityIntent, RevalidateEntityIntent, RollbackEffect, RollbackOutcome, RollbackSummary,
    SavepointId, SelectedBranchRootDenialReason, TransactionCommitError, TransactionId, UndoRecord,
    UpdateEntityFieldsIntent, UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
pub use crate::validation::data::{
    CustomInvariantFailureIdentity, CustomInvariantFailurePhase, CustomInvariantRuleId,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, InvariantViolationFields,
    ResultCustomInvariantFailureKind,
};
pub use worth_foundational::facade::AspectFieldLocator;
