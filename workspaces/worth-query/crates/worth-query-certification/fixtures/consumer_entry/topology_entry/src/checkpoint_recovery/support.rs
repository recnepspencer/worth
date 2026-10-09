mod execution_policy;
#[cfg(feature = "test-query-execution-observer")]
pub(super) use execution_policy::CHECKPOINT_EXECUTION_POLICY;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationContributionTuple,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    },
    runtime::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
        RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
        RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
        RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
        WorthQueryApplicationCandidateResourceProfile, WorthQueryApplicationQueryResourceProfile,
        WorthQueryProductWorldClock, WorthQueryProductWorldResources,
    },
};

use super::*;
pub(super) mod capacity_region;
mod installation;
pub(super) mod retained_inventory;

pub(super) type Application = application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    CheckpointProgram,
>;

pub(super) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
) -> Application {
    install_with_demand_profile(checkpoint, Default::default())
}

pub(super) fn install_with_domain_denial(domain_denial: Arc<AtomicBool>) -> Application {
    installation::try_install_program_with_limits_and_domain_denial::<CheckpointProgram>(
        None,
        Default::default(),
        // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
        limits(32, invalidation(128 * 1_024 * 1_024, 1_000_000, 32)),
        seed_cycle,
        domain_denial,
    )
    .expect("the checkpoint source installs")
}

pub(super) fn install_with_demand_profile(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
) -> Application {
    install_program::<CheckpointProgram>(checkpoint, profile)
}

pub(super) fn install_program<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    install_program_with_history::<Program>(checkpoint, profile, 32, 128 * 1_024 * 1_024)
}

pub(super) fn install_program_with_history<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
    retained_invalidation_bytes: u64,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    install_program_with_seed::<Program>(
        checkpoint,
        profile,
        retained_composite_commits,
        retained_invalidation_bytes,
        1_000_000,
        seed_cycle,
    )
}

pub(super) fn install_program_with_seed<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
    retained_invalidation_bytes: u64,
    maximum_invalidation_work: u64,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    // The invalidation window keeps no more positions than the World retains.
    let kept = usize::try_from(retained_composite_commits.min(128)).unwrap();
    let limits = limits(
        retained_composite_commits,
        invalidation(retained_invalidation_bytes, maximum_invalidation_work, kept),
    );
    install_program_with_limits::<Program>(checkpoint, profile, limits, seed)
}

/// Installs with caller-built limits, so a test can keep the invalidation
/// resources it observes.
pub(super) fn install_program_with_limits<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let expectation = match checkpoint {
        Some(_) => "the checkpoint restores",
        None => "the checkpoint source installs",
    };
    try_install_program_with_limits::<Program>(checkpoint, profile, limits, seed)
        .expect(expectation)
}

/// Installs or restores, returning the denial when the installation is refused.
pub(super) fn try_install_program_with_limits<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>,
    Box<application_installation::WorthQueryInMemoryApplicationDenial>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    installation::try_install_program_with_limits_and_domain_denial::<Program>(
        checkpoint,
        profile,
        limits,
        seed,
        Arc::new(AtomicBool::new(false)),
    )
}

pub(super) fn seed_cycle<Schema: TopologySchemaBinding>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) {
    for (name, x, y) in [
        ("a", 1, 1),
        ("b", 10, 1),
        ("c", 1, 10),
        ("isolated", 50, 50),
        ("island", 60, 50),
        ("atoll", 50, 60),
    ] {
        let key = format!("anchor-{name}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Body::reference::<Schema>(), entity_key(&key))
                    .field(BodyKey::reference::<Schema>(), key)
                    // The initial performed publication writes an equal derived
                    // value; native revisions still decide whether it can reuse.
                    .field(Length::reference::<Schema>(), length(y + 1))
                    .field(PositionX::reference::<Schema>(), length(x))
                    .field(PositionY::reference::<Schema>(), length(y)),
            )
            .unwrap();
    }
    for (from, to) in [
        ("a", "b"),
        ("b", "c"),
        ("c", "a"),
        ("isolated", "island"),
        ("island", "atoll"),
        ("atoll", "isolated"),
    ] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<Schema>(),
                format!("anchor-{from}-to-{to}"),
                entity_key(&format!("anchor-{from}")),
                entity_key(&format!("anchor-{to}")),
            ))
            .unwrap();
    }
}

fn entity_key<Schema: TopologySchemaBinding>(
    key: &str,
) -> WorthQueryApplicationEntityKey<Schema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

pub(super) fn length(value: u64) -> PositiveLength {
    PositiveLength::new(value).unwrap()
}

pub(super) fn invalidation(
    retained_invalidation_bytes: u64,
    maximum_invalidation_work: u64,
    retained_positions: usize,
) -> worth_query_host::facade::runtime::WorthQueryInvalidationResources {
    worth_query_host::facade::runtime::WorthQueryInvalidationResources::install(
        worth_query_host::facade::runtime::WorthQueryInvalidationResourceInstallation::bounded(
            maximum_invalidation_work,
            64 * 1_024 * 1_024,
            retained_invalidation_bytes,
            retained_positions,
        ),
    )
    .expect("the Query invalidation installation is valid")
}

pub(super) fn limits(
    retained_composite_commits: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) -> WorthQueryInMemoryApplicationLimits {
    let host = candidates();
    limits_with_room(retained_composite_commits, 16, 64, invalidation, host)
}

/// The candidate resources of a host whose widest operation is the default.
pub(super) fn candidates() -> WorthQueryApplicationCandidateResourceProfile {
    WorthQueryApplicationCandidateResourceProfile::physical_resources(4_096, 8_192).unwrap()
}

/// A world with room for a long history under open demands. Rows settled at
/// different commits keep their own product observations active, and every
/// open demand pins the exact components it settled on.
pub(super) fn limits_with_room(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    candidates: WorthQueryApplicationCandidateResourceProfile,
) -> WorthQueryInMemoryApplicationLimits {
    limits_with_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        candidates,
        execution_policy::CHECKPOINT_EXECUTION_POLICY,
    )
}

pub(super) fn limits_with_policy(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    candidates: WorthQueryApplicationCandidateResourceProfile,
    policy: worth_foundational::ExecutionRequestPolicy,
) -> WorthQueryInMemoryApplicationLimits {
    limits_with_history_and_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        candidates,
        524_288,
        policy,
    )
}

pub(super) fn limits_with_history_room(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    candidates: WorthQueryApplicationCandidateResourceProfile,
    history_metadata_bytes: u64,
) -> WorthQueryInMemoryApplicationLimits {
    limits_with_history_and_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        candidates,
        history_metadata_bytes,
        execution_policy::CHECKPOINT_EXECUTION_POLICY,
    )
}

pub(super) fn limits_with_history_and_policy(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    candidates: WorthQueryApplicationCandidateResourceProfile,
    history_metadata_bytes: u64,
    policy: worth_foundational::ExecutionRequestPolicy,
) -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        WorthQueryProductWorldResources::install(
            RuntimeWorldBudgetInstallation {
                branches: RuntimeWorldBranchBudgetInstallation {
                    live_product_branches: 4,
                },
                history: RuntimeWorldHistoryBudgetInstallation {
                    retained_composite_commits,
                    history_metadata_bytes,
                },
                observations: RuntimeWorldObservationBudgetInstallation {
                    active_observations,
                },
                publication: RuntimeWorldPublicationBudgetInstallation {
                    active_publication_attempts: 4,
                },
                recovery: RuntimeWorldRecoveryBudgetInstallation {
                    retained_product_unpublished_records: 4,
                    retained_partial_metadata_bytes: 524_288,
                },
                retention: RuntimeWorldRetentionBudgetInstallation {
                    unique_exact_component_pins,
                    in_flight_pin_acquisition_reservations: 16,
                },
                custody: RuntimeWorldCustodyBudgetInstallation {
                    owner_created_component_custody_records: 16,
                },
            },
            WorthQueryProductWorldClock::start(),
            invalidation,
            policy,
        )
        .unwrap(),
        candidates,
        WorthQueryApplicationQueryResourceProfile::bounded(4_096, 4_096, 4_096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

mod authentication_fixture;
pub(super) use authentication_fixture::authenticate;
pub(super) use authentication_fixture::external_identity;
