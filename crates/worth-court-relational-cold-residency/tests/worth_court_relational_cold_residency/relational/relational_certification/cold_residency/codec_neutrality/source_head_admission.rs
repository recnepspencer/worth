//! Codec adoption cannot leave a second current source-contract branch alive.
use super::fixture::*;

#[test]
fn codec_only_restore_refuses_partial_branch_adoption_while_exact_restore_keeps_both_heads() {
    let request = super::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let sibling = worth_relational::facade::history::BranchId("retained-source-head".into());
    let (_, source) = world
        .runtime
        .observe_fork_source(world.runtime.main_branch_identity().branch_id())
        .unwrap();
    world
        .runtime
        .fork_branch(execution, sibling, source)
        .unwrap();
    let checkpoint = capture(execution, &world.runtime);
    for target_codec in [1, 2] {
        let (mut target, _) = installed(target_codec);
        let initial_identity = target.runtime_instance_id();
        let outcome = target
            .durability_recovery()
            .restore_native_checkpoint_for_codec_transition_with_authority(execution, &checkpoint, worth_testkit_relational::cold_recovery::members_source(&checkpoint));
        if target_codec == 1 {
            let (_, recovered) = outcome.unwrap();
            assert!(!recovered.codec_transition_pending());
            assert_eq!(
                super::super::selected_directory::read_native(&capture(execution, &target))
                    .branch_cells
                    .len(),
                2
            );
        } else {
            assert!(outcome.is_err());
            assert_eq!(target.runtime_instance_id(), initial_identity);
            assert!(target.history().historical_latest_commit().is_none());
        }
    }
}
