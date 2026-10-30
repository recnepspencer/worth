//! Native checkpoint-fixture installation with caller-owned limits and seed.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

/// Returns installation refusals while preserving caller-owned domain control.
pub(super) fn try_install_program_with_limits_and_domain_denial<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    domain_denial: Arc<AtomicBool>,
    producer_contacts: Option<crate::producer::ProviderContacts>,
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
    let configuration = configuration(domain_denial, producer_contacts);
    install_configured::<Program>(checkpoint, profile, limits, seed, configuration)
}

fn configuration(
    domain_denial: Arc<AtomicBool>,
    producer_contacts: Option<crate::producer::ProviderContacts>,
) -> TopologyConfiguration {
    TopologyConfiguration {
        final_feature: crate::planar_final_feature_runtime(),
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        invariant_read_observer: None,
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: domain_denial,
        producer_contacts,
    }
}

/// The fixture caller supplies its original setup phase, separately from World policy.
pub(in crate::checkpoint_recovery) fn install_program_in_bootstrap<Program>(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
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
    install_configured_in_bootstrap::<Program>(
        phase,
        profile,
        limits,
        seed,
        configuration(Arc::new(AtomicBool::new(false)), None),
    )
}

/// The same installer accepts the contribution's actual observer configuration.
pub(super) fn install_configured<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
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
    let source = match checkpoint {
        Some(checkpoint) => InstallSource::Checkpoint(checkpoint),
        None => InstallSource::Fresh,
    };
    install_configured_source::<Program>(source, profile, limits, seed, configuration)
}

pub(in crate::checkpoint_recovery) fn install_configured_in_bootstrap<Program>(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
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
    install_configured_source::<Program>(
        InstallSource::Bootstrap(phase),
        profile,
        limits,
        seed,
        configuration,
    )
}

enum InstallSource<'borrow, 'phase> {
    Fresh,
    Checkpoint(application_installation::WorthQueryApplicationCheckpoint),
    Bootstrap(&'borrow worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'phase>),
}

fn install_configured_source<Program>(
    source: InstallSource<'_, '_>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
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
    let configuration = (configuration,);
    let declaration = CheckpointSchema::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<CheckpointSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    let initial_state = |_phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>, graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>, installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<CheckpointSchema>| {
        bind_model_owner(graph, installed)?;
        seed(graph);
        Ok(())
    };
    match source {
        InstallSource::Checkpoint(checkpoint) => {
            application_installation::in_memory_program_from_checkpoint(
                program,
                declaration,
                configuration,
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                limits.with_output_demand_resources(profile),
                checkpoint,
            )
        }
        InstallSource::Bootstrap(phase) => {
            application_installation::in_memory_rostered_program_in_advancement(
                phase,
                program,
                application_installation::WorthQueryApplicationProgramRoster::new(),
                declaration,
                configuration,
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                limits.with_output_demand_resources(profile),
                initial_state,
            )
        }
        InstallSource::Fresh => application_installation::in_memory_program(
            program,
            declaration,
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            limits.with_output_demand_resources(profile),
            initial_state,
        ),
    }
    .map_err(Box::new)
}
