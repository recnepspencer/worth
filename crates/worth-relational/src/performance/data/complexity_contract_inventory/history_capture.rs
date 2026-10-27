use super::{ComplexityContract, ComplexityStatus};

pub(super) const BOUNDED_FOLD: ComplexityContract = ComplexityContract {
    id: "runtime.checkpoint.history_consolidation",
    function_path: "storage/residency/{checkpoint_capture.rs,history_capture_bound.rs,history_fold.rs}",
    declared_time_complexity: "O(encoded_rows + dependency_directory_edges * log(dependency_members)); consolidation additionally O(Full_fold_input + expanded_history + symbol_admission + flat_image_encoding)",
    budget_summary: "Checkpoint output has at most eight dependency layers. Layered distinct encoded fold input is at most 64 MiB, or capture demands Full and writes one flat image before publication. Intrinsic flat history size remains subject to the 1 GiB Full input/output limits. These bounds are independent of restart count; threshold maintenance is explicit capture I/O, while below-threshold capture performs no region loads.",
    status: ComplexityStatus::Verified,
    proof_tests: &[
        "storage::residency::tests::history_consolidation_tests::restart_cycles_keep_full_history_input_bounded_with_headroom",
        "storage::residency::tests::history_consolidation_tests::consolidation_damage_is_sticky_and_a_stopped_capture_is_retryable",
        "tests::durability::medium::cold_members::checkpoint_restart_bound::checkpoint_restart_cycles_preserve_full_reads_and_retention",
        "storage::residency::tests::layered_history_tests::head_capture_acknowledges_and_carries_the_entire_layer_closure_without_loading",
    ],
};
