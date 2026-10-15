use super::super::support::collect_subscriber_patches_from_head;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::tests::harness::certify::assertions::{
    assert_multi_subscriber_converges, assert_visible_truth_matches,
};
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;
use crate::tests::harness::model::truth_model::VisibleTruthSummary;
use crate::tests::harness::observe::patch_stream::collect_patch_stream_from_head;
use crate::tests::harness::observe::subscriber_stream::{
    collect_multi_subscriber_views, random_checkpoints_from_patches,
};
use crate::tests::harness::scenario::operation::ScenarioOperation;
use crate::tests::harness::scenario::runner::{build_property_runtime, run_property_scenario};
use crate::tests::support::*;
use proptest::collection::vec;
use proptest::prelude::*;

prop_compose! {
    fn arb_scenario_operation()(
        tag in 0usize..9,
        a in 0usize..16,
        b in 0usize..16,
        _c in 0usize..16,
        partition in 0usize..4,
    ) -> ScenarioOperation {
        let partition = match partition {
            0 => PartitionId(7),
            1 => PartitionId(11),
            2 => PartitionId(29),
            _ => PartitionId(31),
        };
        match tag {
            0 => ScenarioOperation::CreateEntity { partition, name: String::new() },
            1 => ScenarioOperation::UpdateEntity { entity_slot: a, name: String::new(), branch_slot: 0 },
            2 => ScenarioOperation::ReplaceEntity { entity_slot: a, name: String::new(), branch_slot: 0, partition },
            3 => ScenarioOperation::DeleteEntity { entity_slot: a, branch_slot: 0 },
            4 => ScenarioOperation::CreateRelation {
                source_slot: a,
                target_slot: b,
                client_key: String::new(),
                partition,
            },
            5 => ScenarioOperation::DeleteRelation { relation_slot: a },
            6 => ScenarioOperation::CaptureSnapshot,
            7 => ScenarioOperation::ReleaseSnapshot { snapshot_slot: a },
            8 => ScenarioOperation::RunRetentionPass,
            _ => ScenarioOperation::DurableCheckpoint,
        }
    }
}

prop_compose! {
    fn arb_branch_operation()(
        tag in 0usize..6,
        a in 0usize..12,
    ) -> ScenarioOperation {
        match tag {
            0 => ScenarioOperation::CreateBranch { branch_name: String::new(), from_branch_slot: 0 },
            1 => ScenarioOperation::UpdateEntity { entity_slot: a, name: String::new(), branch_slot: a },
            2 => ScenarioOperation::MergeBranchIntoMain { branch_slot: a },
            3 => ScenarioOperation::CaptureSnapshot,
            4 => ScenarioOperation::ReleaseSnapshot { snapshot_slot: a },
            5 => ScenarioOperation::RunRetentionPass,
            _ => ScenarioOperation::DurableCheckpoint,
        }
    }
}

prop_compose! {
    fn arb_windows()(windows in vec(1usize..10, 3..8)) -> Vec<usize> {
        let mut windows = windows;
        windows.sort_unstable();
        windows.dedup();
        windows
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_global_rejects: 4096,
        .. ProptestConfig::default()
    })]

    #[test]
    fn cdc_property_random_operation_matrix_converges(
        operations in vec(arb_scenario_operation(), 24..96),
        windows in arb_windows(),
        checkpoint_seed in any::<u64>(),
    ) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let world = run_property_scenario(execution,
            operations.clone(),
            RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
        );
        let replay_world = run_property_scenario(execution,
            operations,
            RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
        );
        let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
        let patch_head = collect_patch_stream_from_head(&world.runtime, 4096);
        prop_assert_eq!(&head, &patch_head);
        prop_assert_eq!(&head, &collect_subscriber_patches_from_head(&replay_world.runtime, 4096));

        let checkpoints = random_checkpoints_from_patches(&head, checkpoint_seed, 12);
        let views = collect_multi_subscriber_views(&world.runtime, &checkpoints, &windows);
        assert_multi_subscriber_converges(
            "property random operation matrix",
            &views,
            &head,
        );

        let world_for_truth = world;
        let truth = VisibleTruthSummary::capture(execution, &world_for_truth.runtime);
        let replay_truth = VisibleTruthSummary::capture(execution, &replay_world.runtime);
        assert_visible_truth_matches(
            "property random operation matrix replay truth",
            &truth,
            &replay_truth,
        );
    }

    #[test]
    fn cdc_property_persisted_random_operation_matrix_recovers(
        operations in vec(arb_scenario_operation(), 16..64),
        checkpoint_seed in any::<u64>(),
    ) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let world = run_property_scenario(execution,
            operations,
            RuntimeHarnessMode::Persisted,
        );
        let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
        let checkpoints = random_checkpoints_from_patches(&head, checkpoint_seed, 8);
        let views = collect_multi_subscriber_views(&world.runtime, &checkpoints, &[1, 2, 4, 8]);
        assert_multi_subscriber_converges(
            "property persisted random operation matrix",
            &views,
            &head,
        );

        world.runtime.durability_authority().checkpoint(execution).unwrap();
        let recovery_plan = world.runtime.durability().recovery_plan(execution, RecoveryMode::NormalRecoveryVerification).unwrap();
        let mut recovered = build_property_runtime(RuntimeHarnessMode::Persisted);
        recovered.durability_recovery().recover(execution, recovery_plan).unwrap();
        let recovered_patch_stream = collect_patch_stream_from_head(&recovered, 4096);
        prop_assert_eq!(recovered_patch_stream, head);
        let truth = VisibleTruthSummary::capture(execution, &world.runtime);
        let recovered_truth = VisibleTruthSummary::capture(execution, &recovered);
        assert_visible_truth_matches(
            "property persisted random operation matrix recovered truth",
            &truth,
            &recovered_truth,
        );
    }

    #[test]
    fn cdc_property_branch_local_update_and_merge_matrix_converges(
        operations in vec(arb_branch_operation(), 16..72),
        checkpoint_seed in any::<u64>(),
    ) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let world = run_property_scenario(execution,
            operations,
            RuntimeHarnessMode::InMemory(RelationalRuntimeProfile::GeometryKernel),
        );
        let head = collect_subscriber_patches_from_head(&world.runtime, 4096);
        let checkpoints = random_checkpoints_from_patches(&head, checkpoint_seed, 8);
        let views = collect_multi_subscriber_views(&world.runtime, &checkpoints, &[1, 2, 4, 8]);
        assert_multi_subscriber_converges(
            "property branch-local update and merge matrix",
            &views,
            &head,
        );
    }
}
