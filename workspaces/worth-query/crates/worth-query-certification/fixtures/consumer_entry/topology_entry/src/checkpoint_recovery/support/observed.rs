//! Explicit provider observations on the same ordinary installation path.
use super::*;
use crate::producer::ProviderContacts;

pub(in super::super) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    contacts: &ProviderContacts,
) -> Application {
    install_program::<CheckpointProgram>(checkpoint, Default::default(), contacts)
}

pub(in super::super) fn install_program<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
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
        checkpoint,
        profile,
        DEFAULT_RETAINED_COMMITS,
        seed_cycle,
        contacts,
    )
}

pub(in super::super) fn install_program_with_seed<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
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
    installation::try_install_program_with_limits_and_domain_denial::<Program>(
        checkpoint,
        profile,
        limits(retained_composite_commits, invalidation()),
        seed,
        Arc::new(AtomicBool::new(false)),
        Some(contacts.clone()),
    )
    .expect("the observed source installs or restores through the ordinary owner")
}

pub(in super::super) fn install_with_domain_denial(
    domain_denial: Arc<AtomicBool>,
    contacts: &ProviderContacts,
) -> Application {
    installation::try_install_program_with_limits_and_domain_denial::<CheckpointProgram>(
        None,
        Default::default(),
        limits(DEFAULT_RETAINED_COMMITS, invalidation()),
        seed_cycle,
        domain_denial,
        Some(contacts.clone()),
    )
    .expect("the observed checkpoint source installs")
}
