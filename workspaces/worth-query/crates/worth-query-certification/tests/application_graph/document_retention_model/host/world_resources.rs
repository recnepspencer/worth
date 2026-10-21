use worth_query_host::facade::runtime;

pub(super) fn world_resources(
    commits: u64,
    metadata_bytes: u64,
    pins: u64,
) -> runtime::WorthQueryProductWorldResources {
    world_resources_with_work(commits, metadata_bytes, pins, 8_000_000)
}

pub(super) const WORKFLOW_SCALE_WIDTH: u64 = 200_000;

pub const ACTIVE_WORLD_OBSERVATIONS: u64 = 128;

pub const WORKFLOW_DEFINITION_BYTES: u32 = 16 * 1024 * 1024;

pub(super) fn workflow_scale_resources() -> runtime::WorthQueryProductWorldResources {
    // These courts protect retained bytes, publication/hash growth and compiler
    // cache reuse, not a fitted work count. The original request still meters
    // all preparation, stops on overflow, and retains its finite memory limit.
    world_resources_with_work(1_024, 256 * 1024 * 1024, 2_048, u64::MAX)
}

pub(super) fn world_resources_with_work(
    commits: u64,
    metadata_bytes: u64,
    pins: u64,
    work: u64,
) -> runtime::WorthQueryProductWorldResources {
    // Bounds this fixture World's serial request memory and deterministic work.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(work)),
    );
    runtime::WorthQueryProductWorldResources::install(
        runtime::RuntimeWorldBudgetInstallation {
            branches: runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: commits,
                history_metadata_bytes: metadata_bytes,
            },
            observations: runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: ACTIVE_WORLD_OBSERVATIONS,
            },
            publication: runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: pins,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        runtime::WorthQueryProductWorldClock::start(),
        runtime::WorthQueryInvalidationResources::new(),
        execution_policy,
    )
    .expect("the document-retention World resources are valid")
}
