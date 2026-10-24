use worth_query_host::facade::runtime;

pub const ACTIVE_WORLD_OBSERVATIONS: u64 = 128;

pub const WORKFLOW_DEFINITION_BYTES: u32 = 16 * 1024 * 1024;

pub(crate) fn world_resources(
    commits: u64,
    metadata_bytes: u64,
    pins: u64,
) -> runtime::WorthQueryProductWorldResources {
    // These ordinary authority courts impose no guessed request-work ceiling.
    // Native checked work and allocation handling retain their existing policy.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
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
