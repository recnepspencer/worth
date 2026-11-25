//! Explicit provider observations on in-memory and Directory Home paths.
use super::*;
use crate::producer::ProviderContacts;
use std::path::PathBuf;

pub(in super::super) fn install(_: Option<()>, contacts: &ProviderContacts) -> Application {
    install_program::<CheckpointProgram>(None, Default::default(), contacts)
}

pub(in super::super) fn install_program<Program>(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    contacts: &ProviderContacts,
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
        None,
        profile,
        DEFAULT_RETAINED_COMMITS,
        seed_cycle,
        contacts,
    )
}

pub(in super::super) fn install_program_with_seed<Program>(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    contacts: &ProviderContacts,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let limits = limits(retained_composite_commits, invalidation());
    installation::try_install_in_memory_configured::<Program>(
        profile,
        limits,
        seed,
        installation::configuration(Arc::new(AtomicBool::new(false)), Some(contacts.clone())),
    )
    .expect("the observed application installs through the ordinary owner")
}

pub(in super::super) fn install_with_domain_denial(
    domain_denial: Arc<AtomicBool>,
    contacts: &ProviderContacts,
) -> Application {
    installation::try_install_in_memory_configured::<CheckpointProgram>(
        Default::default(),
        limits(DEFAULT_RETAINED_COMMITS, invalidation()),
        seed_cycle,
        installation::configuration(domain_denial, Some(contacts.clone())),
    )
    .expect("the observed checkpoint source installs")
}

pub(in super::super) fn open_directory(
    directory: Option<PathBuf>,
    contacts: &ProviderContacts,
) -> DirApplication {
    open_directory_program::<CheckpointProgram>(directory, Default::default(), contacts)
}

pub(in super::super) fn open_directory_with_limits(
    directory: Option<PathBuf>,
    limits: WorthQueryApplicationLimits,
    contacts: &ProviderContacts,
) -> DirApplication {
    installation::try_open_directory_program::<CheckpointProgram>(
        directory,
        Default::default(),
        limits,
        seed_cycle,
        Arc::new(AtomicBool::new(false)),
        Some(contacts.clone()),
    )
    .expect("the observed Directory Home opens under its original finite request policy")
}

pub(in super::super) fn open_directory_program<Program>(
    directory: Option<PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    contacts: &ProviderContacts,
) -> DirectoryApp<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    open_directory_program_with_seed::<Program>(
        directory,
        profile,
        DEFAULT_RETAINED_COMMITS,
        seed_cycle,
        contacts,
    )
}

pub(in super::super) fn open_directory_program_with_seed<Program>(
    directory: Option<PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    contacts: &ProviderContacts,
) -> DirectoryApp<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let limits = limits(retained_composite_commits, invalidation());
    installation::try_open_directory_program::<Program>(
        directory,
        profile,
        limits,
        seed,
        Arc::new(AtomicBool::new(false)),
        Some(contacts.clone()),
    )
    .expect("the observed Directory Home opens through the ordinary owner")
}
