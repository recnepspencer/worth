use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

// Bounds this checkpoint fixture World's serial request memory and work.
pub(crate) const CHECKPOINT_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(
        std::num::NonZeroUsize::MIN,
        64 * 1024 * 1024,
        Some(8_000_000),
    ),
);

/// Complete currentness includes retained output and original discovery facts.
/// These semantic courts admit that work under the fixture's finite request
/// policy; exact work-limit boundaries are exercised by the native meter courts.
pub(in crate::checkpoint_recovery) fn currentness_work_limit() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(
        usize::try_from(
            CHECKPOINT_EXECUTION_POLICY
                .budget()
                .work_ceiling()
                .expect("checkpoint fixture explicitly chooses a finite request"),
        )
        .expect("fixture request work fits the host word"),
    )
    .expect("fixture request work is positive")
}
