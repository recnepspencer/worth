//! Directory Home and bootstrap fixture installation with caller-owned limits and seed.
use super::*;
use std::path::PathBuf;
use worth_query_host::facade::{
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeRequest,
};

pub(super) fn try_open_directory_program<Program>(
    directory: Option<PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    domain_denial: Arc<AtomicBool>,
    producer_contacts: Option<crate::producer::ProviderContacts>,
) -> Result<
    DirectoryApp<CheckpointSchema, Program>,
    Box<WorthQueryApplicationHomeOpenRefused<CheckpointSchema, Program>>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    open_directory_configured_with_build::<Program>(
        directory,
        profile,
        limits,
        seed,
        configuration(domain_denial, producer_contacts),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
    )
}

pub(super) fn open_directory_configured_with_build<Program>(
    directory: Option<PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
    build_identity: ApplicationBuildIdentity,
) -> Result<
    DirectoryApp<CheckpointSchema, Program>,
    Box<WorthQueryApplicationHomeOpenRefused<CheckpointSchema, Program>>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let dir = directory.unwrap_or_else(|| directory::unique_directory("topology"));
    let home = directory::with_directory_bootstrap(|phase| {
        open_directory_in_bootstrap_with_build::<Program>(
            phase,
            dir.clone(),
            profile,
            limits,
            seed,
            configuration,
            build_identity,
        )
    })?;
    Ok(home)
}

pub(super) fn open_directory_in_bootstrap<Program>(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
    directory: PathBuf,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
) -> Result<
    DirectoryApp<CheckpointSchema, Program>,
    Box<WorthQueryApplicationHomeOpenRefused<CheckpointSchema, Program>>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    open_directory_in_bootstrap_with_build::<Program>(
        phase,
        directory,
        profile,
        limits,
        seed,
        configuration,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
    )
}

pub(super) fn open_directory_in_bootstrap_with_build<Program>(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
    directory: PathBuf,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
    build_identity: ApplicationBuildIdentity,
) -> Result<
    DirectoryApp<CheckpointSchema, Program>,
    Box<WorthQueryApplicationHomeOpenRefused<CheckpointSchema, Program>>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let declaration = <CheckpointSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<CheckpointSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    let home = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory.clone()),
        program,
        application_installation::WorthQueryApplicationProgramRoster::new(),
        declaration,
        (configuration,),
        build_identity,
        limits.with_output_demand_resources(profile),
        |_phase, graph, installed| {
            bind_model_owner(graph, installed)?;
            seed(graph);
            Ok(())
        },
    )
    .with_authorization_time_source(
        worth_query_host::facade::primary_graph::WorthQueryRuntimeClock::system()
            .with_admission_clock(authentication::WorthQueryAdmissionClock::system()),
    )
    .open_in_advancement(phase)
    .map_err(Box::new)?;
    Ok(DirectoryApp::wrap(home, directory))
}

pub(super) fn configuration(
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
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>,
    Box<application_installation::WorthQueryApplicationPreparationDenial>,
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

pub(in crate::checkpoint_recovery) fn install_configured_in_bootstrap<Program>(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>,
    Box<application_installation::WorthQueryApplicationPreparationDenial>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let declaration = <CheckpointSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<CheckpointSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    application_installation::in_memory_rostered_program_in_advancement(
        phase,
        program,
        application_installation::WorthQueryApplicationProgramRoster::new(),
        declaration,
        (configuration,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits.with_output_demand_resources(profile),
        |_phase, graph, installed| {
            bind_model_owner(graph, installed)?;
            seed(graph);
            Ok(())
        },
    )
    .map_err(Box::new)
}

pub(in crate::checkpoint_recovery) fn install_on_clock(
    clock: authentication::WorthQueryAdmissionClock,
) -> Application {
    let limits = limits_with_history_and_policy(
        DEFAULT_RETAINED_COMMITS,
        16,
        64,
        invalidation(),
        524_288,
        execution_policy::CHECKPOINT_EXECUTION_POLICY,
        clock.clone(),
    );
    let declaration = <CheckpointSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<CheckpointSchema, CheckpointProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    application_installation::in_memory_program_with_authorization_time_source(
        program,
        declaration,
        (configuration(Arc::new(AtomicBool::new(false)), None),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits,
        |_phase, graph, installed| {
            bind_model_owner(graph, installed)?;
            seed_cycle(graph);
            Ok(())
        },
        worth_query_host::facade::primary_graph::WorthQueryRuntimeClock::system()
            .with_admission_clock(clock),
    )
    .expect("the checkpoint source installs on its owned clock")
}

pub(super) fn try_install_in_memory_program_with_limits<Program>(
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>,
    Box<application_installation::WorthQueryApplicationPreparationDenial>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    try_install_in_memory_configured::<Program>(
        profile,
        limits,
        seed,
        configuration(Arc::new(AtomicBool::new(false)), None),
    )
}

pub(super) fn try_install_in_memory_configured<Program>(
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    configuration: TopologyConfiguration,
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>,
    Box<application_installation::WorthQueryApplicationPreparationDenial>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let declaration = <CheckpointSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<CheckpointSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    application_installation::in_memory_program_with_authorization_time_source(
        program,
        declaration,
        (configuration,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits.with_output_demand_resources(profile),
        |_phase, graph, installed| {
            bind_model_owner(graph, installed)?;
            seed(graph);
            Ok(())
        },
        worth_query_host::facade::primary_graph::WorthQueryRuntimeClock::system()
            .with_admission_clock(authentication::WorthQueryAdmissionClock::system()),
    )
    .map_err(Box::new)
}
