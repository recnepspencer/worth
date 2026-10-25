//! Codec adoption cannot leave a second current source-contract branch alive.
use super::fixture::*;

#[test]
fn codec_only_restore_refuses_partial_branch_adoption_while_exact_restore_keeps_both_heads() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    world
        .runtime
        .history_authority()
        .fork_branch_from(
            execution,
            crate::history::data::BranchId("retained-source-head".into()),
            world.runtime.main_branch_identity().branch_id(),
        )
        .unwrap();
    let checkpoint = capture(&world.runtime);
    for target_codec in [1, 2] {
        let (mut target, _) = installed(target_codec);
        let initial_identity = target.runtime_instance_id();
        let outcome = target
            .durability_recovery()
            .restore_native_checkpoint_for_codec_transition_with_authority(execution, &checkpoint);
        if target_codec == 1 {
            let (_, recovered) = outcome.unwrap();
            assert!(!recovered.codec_transition_pending());
            assert_eq!(target.history().branch_cells_snapshot().len(), 2);
        } else {
            assert!(outcome.is_err());
            assert_eq!(target.runtime_instance_id(), initial_identity);
            assert!(target.history().latest_commit().is_none());
        }
    }
}
