//! Explicit provider observations on the same ordinary installation path.
use super::*;
use crate::producer::ProviderContacts;

pub(in super::super) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    contacts: &ProviderContacts,
) -> Application {
    install_program::<CheckpointProgram>(checkpoint, Default::default(), contacts)
}

pub(in super::super) fn install_with_limits(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    limits: WorthQueryApplicationLimits,
    contacts: &ProviderContacts,
) -> Application {
    match checkpoint {
        Some(checkpoint) => {
            worth_query_host::facade::application_contribution::with_bootstrap_advancement(
                super::execution_policy::POPULATION_BOOTSTRAP_POLICY,
                |phase| {
                    installation::install_checkpoint_in_bootstrap::<CheckpointProgram>(
                        &phase,
                        checkpoint,
                        Default::default(),
                        limits,
                        installation::configuration(
                            Arc::new(AtomicBool::new(false)),
                            Some(contacts.clone()),
                        ),
                    )
                },
            )
            .expect("the observed reconstruction owns its original opening request")
            .expect("the observed restore retains its original finite serving policy")
        }
        None => {
            installation::try_install_program_with_limits_and_domain_denial::<CheckpointProgram>(
                None,
                Default::default(),
                limits,
                seed_cycle,
                Arc::new(AtomicBool::new(false)),
                Some(contacts.clone()),
            )
            .expect("the observed source installs under its original finite request policy")
        }
    }
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
    let limits = limits(retained_composite_commits, invalidation());
    match checkpoint {
        Some(checkpoint) => {
            worth_query_host::facade::application_contribution::with_bootstrap_advancement(
                super::execution_policy::POPULATION_BOOTSTRAP_POLICY,
                |phase| {
                    installation::install_checkpoint_in_bootstrap::<Program>(
                        &phase,
                        checkpoint,
                        profile,
                        limits,
                        installation::configuration(
                            Arc::new(AtomicBool::new(false)),
                            Some(contacts.clone()),
                        ),
                    )
                },
            )
            .expect("the observed reconstruction owns its original opening request")
            .expect("the observed checkpoint restores with its actual provider observer")
        }
        None => installation::try_install_program_with_limits_and_domain_denial::<Program>(
            None,
            profile,
            limits,
            seed,
            Arc::new(AtomicBool::new(false)),
            Some(contacts.clone()),
        )
        .expect("the observed source installs through the ordinary owner"),
    }
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
