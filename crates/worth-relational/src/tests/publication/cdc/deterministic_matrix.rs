use super::support::{collect_subscriber_patches, run_seeded_cdc_scenario};
use crate::tests::harness::certify::assertions::assert_window_matrix_matches;

#[test]
fn seeded_cdc_matrix_is_deterministic_and_window_stable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in 0_u64..12 {
        let left = run_seeded_cdc_scenario(execution, seed, 48);
        let right = run_seeded_cdc_scenario(execution, seed, 48);

        let full_left =
            collect_subscriber_patches(&left.runtime, left.baseline_checkpoint.clone(), 256);
        let full_right =
            collect_subscriber_patches(&right.runtime, right.baseline_checkpoint.clone(), 256);

        assert_eq!(full_left, full_right, "seed {seed} diverged");

        let window_matrix = [1_usize, 2, 3, 5, 8]
            .into_iter()
            .map(|window_size| {
                (
                    window_size,
                    collect_subscriber_patches(
                        &left.runtime,
                        left.baseline_checkpoint.clone(),
                        window_size,
                    ),
                )
            })
            .collect::<Vec<_>>();
        assert_window_matrix_matches(
            &format!("seed {seed} deterministic matrix"),
            &full_left,
            &window_matrix,
        );
    }
}
