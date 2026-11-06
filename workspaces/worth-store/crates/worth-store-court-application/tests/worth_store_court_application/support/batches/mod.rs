#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;

/// Recompute the actual authored batch bodies used by the transition courts.
pub(crate) fn transition_body_roster(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    steps: u32,
) -> Vec<worth_foundational::facade::HomeProgramTransitionBodyDigest> {
    (0..steps)
        .map(|step| {
            worth_relational::facade::runtime::home_program_transition_body_digest(
                request,
                &worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                &[create(step as usize)],
            )
            .unwrap()
        })
        .collect()
}
