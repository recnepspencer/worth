use std::collections::BTreeSet;
use std::sync::Arc;

use crate::capabilities::DurabilityRead;
use crate::commit_strategies::data::StrategyCommitArtifactBundle;
use crate::commit_strategies::strategies::{
    IntentReconciliationInput, IntentReconciliationStrategy, ReplicaConvergenceInput,
    ReplicaConvergenceStrategy,
};
use crate::facade::commit_strategies::NativeStrategyCommitRequest;
use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::durability::{DurabilityMode, DurableStoreLayout};
use crate::facade::history::{BranchId, RelationalCommitReceipt};
use crate::facade::merge::{MergeIntent, MergePlanningRequest};
use crate::facade::replay::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayObservableSurface,
    ReplayVerificationMode,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::transactions::CommitResult;
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity, entity_field_aspect,
    entity_u64_field_aspect, lifecycle_aspect, read_entity_name, unique_test_store_path,
    AspectSchemaFixture,
};
use crate::transactions::data::AspectFieldPatch;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
    LocatorAuthority,
};

mod aspect_visibility;
mod certification_bundle;
mod field_patch_fixtures;
mod final_recovery;
mod planning_assertions;
mod replay_recovery;
mod stage_sequence;
mod strategy_runtime;

use aspect_visibility::*;
use certification_bundle::*;
use field_patch_fixtures::*;
use planning_assertions::*;
use replay_recovery::*;
use strategy_runtime::*;
#[test]
pub(super) fn milestone_8_5_kubernetes_style_intent_commit_certification_proves_staged_controller_outcomes(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let certification = stage_sequence::run_kubernetes_style_certification(execution);
    assert!(!certification.overlap_conflict.conflict.records.is_empty());
    assert!(!certification
        .narrowed_non_conflict
        .conflict
        .records
        .is_empty());
    assert!(!certification
        .rebroadened_conflict
        .conflict
        .records
        .is_empty());
    assert!(!certification
        .revalidated_converged_overlap
        .lowered_plan
        .records
        .is_empty());
    assert_eq!(
        certification.revalidation_noop.changed_record_count,
        certification.revalidation_noop.patch_record_count
    );
    assert_strategy_replay_clean(&certification.broad_intent_replay, "certified broad intent");
    assert_strategy_replay_clean(
        &certification.first_converge_replay,
        "certified first converge",
    );
    assert_strategy_replay_clean(
        &certification.rebroadened_intent_replay,
        "certified rebroadened intent",
    );
    assert_strategy_replay_clean(
        &certification.revalidation_noop_replay,
        "certified revalidation",
    );
    assert!(certification.branch_heads.main.is_some());
    assert!(certification.branch_heads.controller.is_some());
    assert_eq!(
        certification.visible_truth.main.entity_name.as_deref(),
        Some("svc-v2")
    );
    assert!(certification
        .visible_truth
        .main
        .replicas_canonical_bytes
        .is_some());
    assert_eq!(
        certification
            .visible_truth
            .controller
            .entity_name
            .as_deref(),
        Some("kube-service")
    );
    assert_eq!(
        certification.visible_truth.main.replicas_canonical_bytes,
        certification
            .visible_truth
            .controller
            .replicas_canonical_bytes
    );
    assert_ne!(
        certification.overlap_conflict,
        certification.narrowed_non_conflict
    );
    assert_ne!(
        certification.narrowed_non_conflict,
        certification.rebroadened_conflict
    );
    assert_ne!(
        certification.rebroadened_conflict,
        certification.revalidated_converged_overlap
    );
}
