use super::*;
/// The chain program with smaller required custody or invalidation retention,
/// and the invalidation resources it retains.
pub(super) fn limited_application(
    required_custody_bytes: usize,
    invalidation_bytes: u64,
    retained_positions: usize,
) -> (
    application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) {
    limited_application_seeded(
        required_custody_bytes,
        invalidation_bytes,
        retained_positions,
        source_world::seed,
    )
}

/// The chain program with smaller required custody or invalidation retention,
/// and the invalidation resources it retains.
pub(super) fn limited_application_seeded(
    required_custody_bytes: usize,
    invalidation_bytes: u64,
    retained_positions: usize,
    seed: impl FnOnce(&mut primary_graph::WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> (
    application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) {
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
            .with_registry_required_retained_bytes(
                std::num::NonZeroUsize::new(required_custody_bytes).unwrap(),
            );
    let invalidation = support::invalidation(
        invalidation_bytes,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        retained_positions,
    );
    let application = support::install_program_with_limits::<program::ChainProgram>(
        None,
        profile,
        support::limits(4_096, invalidation.clone()),
        seed,
    );
    (application, invalidation)
}
