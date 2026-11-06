mod runtime_time;
pub(super) use runtime_time::RuntimeTime;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::runtime::RuntimeWorldBranchBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldCustodyBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldHistoryBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldObservationBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldPublicationBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldRecoveryBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldRetentionBudgetInstallation;
use worth_query_host::facade::runtime::WorthQueryInvalidationResources;
use worth_query_host::facade::runtime::WorthQueryProductWorldClock;
use worth_query_host::facade::runtime::WorthQueryProductWorldResources;

pub(super) const ACTIVE_WORLD_OBSERVATIONS: u64 = 32;

// Bounds this fixture World's serial request memory and deterministic work.
pub(super) const EXECUTION_POLICY: worth_foundational::ExecutionRequestPolicy =
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );

pub(super) fn world_resources() -> WorthQueryProductWorldResources {
    world_resources_for(
        EXECUTION_POLICY,
        ACTIVE_WORLD_OBSERVATIONS,
        8,
        WorthQueryAdmissionClock::system(),
    )
}

fn world_resources_for(
    policy: worth_foundational::ExecutionRequestPolicy,
    active_observations: u64,
    branch_capacity: u64,
    clock: WorthQueryAdmissionClock,
) -> WorthQueryProductWorldResources {
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: branch_capacity,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 64,
                history_metadata_bytes: 1_048_576,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 8,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 8,
                retained_partial_metadata_bytes: 1_048_576,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 128,
                in_flight_pin_acquisition_reservations: 32,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 32,
            },
        },
        WorthQueryProductWorldClock::start(clock),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .expect("the consumer's finite World resources are valid")
}

pub(super) fn application_limits(
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    application_limits_for_world(world_resources())
}

pub(super) fn application_limits_with_branches(
    branch_capacity: u64,
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    application_limits_for_world(world_resources_for(
        EXECUTION_POLICY,
        ACTIVE_WORLD_OBSERVATIONS,
        branch_capacity,
        WorthQueryAdmissionClock::system(),
    ))
}

pub(super) fn application_limits_with_clock(
    policy: worth_foundational::ExecutionRequestPolicy,
    clock: WorthQueryAdmissionClock,
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    application_limits_for_world(world_resources_for(
        policy,
        ACTIVE_WORLD_OBSERVATIONS,
        8,
        clock,
    ))
}

pub(super) fn application_limits_with_observations(
    policy: worth_foundational::ExecutionRequestPolicy,
    active_observations: u64,
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    application_limits_for_world(world_resources_for(
        policy,
        active_observations,
        8,
        WorthQueryAdmissionClock::system(),
    ))
}

fn application_limits_for_world(
    world: WorthQueryProductWorldResources,
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits::new(
        world,
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(32).expect("concurrent graph work is non-zero"),
        ),
        worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
