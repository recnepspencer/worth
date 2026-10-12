use crate::commit_strategies::data::{
    CommitStrategyExecutionRegistration, CommitStrategyExecutor, CommitStrategyRegistration,
    StrategyCommitArtifactBundle, StrategyExecutionResult, StrategyExecutorFailure,
    StrategyExecutorFailureClass, StrategyObservationContext,
};
use crate::commit_strategies::strategies::{
    AspectFieldReconciliationInput, AspectFieldReconciliationStrategy,
    EntityReplacementReconciliationInput, EntityReplacementReconciliationStrategy,
    IntentReconciliationInput, IntentReconciliationStrategy, ReplicaConvergenceInput,
    ReplicaConvergenceStrategy,
};
use crate::facade::commit_strategies::NativeStrategyCommitRequest;
use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::durability::{DurabilityMode, DurableStoreLayout};
use crate::facade::history::{BranchId, RelationalCommitReceipt};
use crate::facade::merge::{MergeIntent, MergePlanningRequest};
use crate::facade::replay::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayMismatchClass,
    ReplayObservableSurface, ReplayVerificationMode,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::tests::support::{
    changed_entities, checkpoint_and_recover_with, create_branch_from_main, create_entity,
    entity_field_aspect, entity_u64_field_aspect, lifecycle_aspect, read_entity_name,
    unique_test_store_path, AspectSchemaFixture,
};
use crate::transactions::data::AspectFieldPatch;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
    LocatorAuthority,
};

mod certification_bundle;
mod controller_sequence;
mod field_patch_fixtures;
mod merge_aspect_conflicts;
mod merge_strategy_conflict;
mod replacement_lineage;
mod replay_recovery;
mod runtime_fixtures;
mod strategy_certification;

use certification_bundle::*;
use field_patch_fixtures::*;
use replacement_lineage::*;
use runtime_fixtures::*;
#[test]
fn milestone_8_5_strategy_certification_preserves_merge_replay_and_recovery_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let certification = strategy_certification::run_strategy_merge_certification(execution);
    assert!(
        certification
            .main_commit_strategy_artifacts
            .lowering_summary()
            .total_intent_count()
            > 0
    );
    assert!(
        certification
            .feature_commit_strategy_artifacts
            .lowering_summary()
            .total_intent_count()
            > 0
    );
    assert!(
        certification
            .replacement
            .replacement_commit_strategy_artifacts
            .lowering_summary()
            .lineage_transition_count()
            > 0
    );
    assert!(!certification.merge_conflict.records.is_empty());
    assert!(!certification.merge_lowered_plan.records.is_empty());
    assert!(!certification
        .aspect_overlap_merge_conflict
        .records
        .is_empty());
    assert!(!certification
        .aspect_overlap_merge_lowered_plan
        .records
        .is_empty());
    assert!(!certification
        .aspect_disjoint_merge_conflict
        .records
        .is_empty());
    assert!(!certification
        .aspect_disjoint_merge_lowered_plan
        .records
        .is_empty());
    assert!(!certification
        .controller_sequence_merge_conflict
        .records
        .is_empty());
    assert!(!certification
        .controller_sequence_merge_lowered_plan
        .records
        .is_empty());
    assert!(certification.main_replay.failure.is_none());
    assert!(certification.feature_replay.failure.is_none());
    assert!(certification
        .main_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(certification
        .feature_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert_eq!(
        certification.controller_sequence_noop.changed_record_count,
        certification.controller_sequence_noop.patch_record_count
    );
    assert!(certification
        .replacement
        .replacement_replay
        .failure
        .is_none());
    assert!(certification
        .replacement
        .replacement_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert_ne!(
        certification.replacement.replacement_lineage.start_lineage,
        certification.replacement.replacement_lineage.end_lineage
    );
    assert!(
        certification
            .missing_executor_replay
            .strategy_surface_mismatch_present
    );
    assert!(
        certification
            .failing_executor_replay
            .strategy_surface_mismatch_present
    );
    assert!(certification.branch_heads.main.is_some());
    assert!(certification.branch_heads.feature.is_some());
    assert_eq!(
        certification.visible_truth.branch_heads,
        certification.branch_heads
    );
    assert_eq!(
        certification.visible_truth.entity_name.as_deref(),
        Some("service-main")
    );
}
