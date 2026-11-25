mod bootstrap_funding;
mod execution_policy;
pub(super) mod observation;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationContributionTuple,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    },
};

use super::*;
mod admission_time;
pub(super) mod directory;
mod installation;
pub(super) use installation::{
    install_configured_in_bootstrap, install_on_clock, install_program_in_bootstrap,
};
pub(super) mod observed;
mod principal_bootstrap;
pub(super) use principal_bootstrap::bind_model_owner;

pub(super) type Application = application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    CheckpointProgram,
>;
pub(super) type DirApplication = DirectoryApp<CheckpointSchema, CheckpointProgram>;

pub(super) const DEFAULT_RETAINED_COMMITS: u64 = 32;

pub(super) fn install(_: Option<()>) -> Application {
    install_with_demand_profile(None, Default::default())
}

pub(super) fn install_with_domain_denial(domain_denial: Arc<AtomicBool>) -> Application {
    installation::try_install_in_memory_configured::<CheckpointProgram>(
        Default::default(),
        limits(DEFAULT_RETAINED_COMMITS, invalidation()),
        seed_cycle,
        installation::configuration(domain_denial, None),
    )
    .expect("the checkpoint source installs")
}

pub(super) fn install_with_demand_profile(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
) -> Application {
    install_program::<CheckpointProgram>(None, profile)
}

pub(super) fn install_program<Program>(
    _: Option<()>,
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
    install_program_with_history::<Program>(None, profile, DEFAULT_RETAINED_COMMITS)
}

pub(super) fn install_program_with_history<Program>(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    install_program_with_seed::<Program>(None, profile, retained_composite_commits, seed_cycle)
}

pub(super) fn install_program_with_seed<Program>(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
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
    let limits = limits(retained_composite_commits, invalidation());
    install_program_with_limits::<Program>(None, profile, limits, seed)
}

pub(super) fn install_program_with_limits<Program>(
    _: Option<()>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
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
    installation::try_install_in_memory_program_with_limits::<Program>(profile, limits, seed)
        .expect("the in-memory application installs")
}

pub(super) fn open_directory(directory: Option<std::path::PathBuf>) -> DirApplication {
    open_directory_program::<CheckpointProgram>(directory, Default::default())
}

pub(super) fn open_directory_program<Program>(
    directory: Option<std::path::PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
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
    )
}

pub(super) fn open_directory_program_with_seed<Program>(
    directory: Option<std::path::PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    retained_composite_commits: u64,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
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
    open_directory_program_with_limits::<Program>(directory, profile, limits, seed)
}

pub(super) fn open_directory_program_with_limits<Program>(
    directory: Option<std::path::PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> DirectoryApp<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    try_open_directory_program_with_limits::<Program>(directory, profile, limits, seed)
        .expect("the Directory Home opens")
}

pub(super) fn open_directory_program_with_limits_and_build<Program>(
    directory: Option<std::path::PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    build_identity: application_installation::ApplicationBuildIdentity,
) -> DirectoryApp<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    installation::open_directory_configured_with_build::<Program>(
        directory,
        profile,
        limits,
        seed,
        installation::configuration(Arc::new(AtomicBool::new(false)), None),
        build_identity,
    )
    .expect("the Directory Home opens with build identity")
}

pub(super) fn try_open_directory_program_with_limits<Program>(
    directory: Option<std::path::PathBuf>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryApplicationLimits,
    seed: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
) -> Result<
    DirectoryApp<CheckpointSchema, Program>,
    Box<worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<CheckpointSchema, Program>>,
>
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    installation::try_open_directory_program::<Program>(
        directory,
        profile,
        limits,
        seed,
        Arc::new(AtomicBool::new(false)),
        None,
    )
}

pub(super) fn try_install_program_with_limits<Program>(
    _checkpoint: Option<()>,
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
    installation::try_install_in_memory_program_with_limits::<Program>(profile, limits, seed)
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
    installation::try_install_in_memory_program_with_limits::<Program>(profile, limits, seed)
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
                    .field(BodyKey::at("BodyKey"), key)
                    // The initial performed publication writes an equal derived
                    // value; native revisions still decide whether it can reuse.
                    .field(Length::at("Length"), length(y + 1))
                    .field(PositionX::at("PositionX"), length(x))
                    .field(PositionY::at("PositionY"), length(y)),
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

mod limits;
pub(super) use limits::{
    invalidation, limits, limits_with_history_and_policy, limits_with_policy, limits_with_room,
};

pub(super) mod authentication_fixture;
pub(super) use authentication_fixture::authenticate;
pub(super) use authentication_fixture::external_identity;

mod observation_policy;
pub(super) use observation_policy::{host_call, retain_read};

pub(in crate::checkpoint_recovery) struct DirectoryApp<Schema: ApplicationSchema, Program: 'static> {
    home: Option<worth_query_host::facade::WorthQueryApplicationHome<Schema, Program>>,
    directory: std::path::PathBuf,
    cleanup_dir: bool,
}
impl<Schema: ApplicationSchema, Program: 'static> DirectoryApp<Schema, Program> {
    pub(in crate::checkpoint_recovery) fn wrap(
        home: worth_query_host::facade::WorthQueryApplicationHome<Schema, Program>,
        directory: std::path::PathBuf,
    ) -> Self {
        Self { home: Some(home), directory, cleanup_dir: true }
    }
    pub(in crate::checkpoint_recovery) fn without_dir_cleanup(mut self) -> Self {
        self.cleanup_dir = false;
        self
    }
    pub(in crate::checkpoint_recovery) fn checkpoint_directory(&self) {
        let home = self.home.as_ref().expect("directory Home is open");
        directory::with_directory_bootstrap(|phase| {
            home.checkpoint_in_advancement(phase)
                .unwrap_or_else(|refused| panic!("directory Home checkpoint refused: {refused:?}"))
        });
    }
    pub(in crate::checkpoint_recovery) fn close_for_reopen(mut self) -> std::path::PathBuf {
        self.cleanup_dir = false;
        let home = self.home.take().expect("directory Home is open");
        directory::with_directory_bootstrap(|phase| {
            home.close_in_advancement(phase)
                .unwrap_or_else(|refused| panic!("directory Home close refused: {refused:?}"))
                .reclaim();
        });
        self.directory.clone()
    }
    pub(in crate::checkpoint_recovery) fn checkpoint_and_close_for_reopen(self) -> std::path::PathBuf {
        self.checkpoint_directory();
        self.close_for_reopen()
    }
}
impl<Schema: ApplicationSchema, Program: 'static> std::ops::Deref for DirectoryApp<Schema, Program> {
    type Target = application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
    fn deref(&self) -> &Self::Target {
        self.home.as_ref().expect("directory Home is open").runtime()
    }
}
impl<Schema: ApplicationSchema, Program: 'static> Drop for DirectoryApp<Schema, Program> {
    fn drop(&mut self) {
        if let Some(home) = self.home.take() {
            directory::with_directory_bootstrap(|phase| {
                if let Ok(closed) = home.close_in_advancement(phase) {
                    closed.reclaim();
                }
            });
        }
        if self.cleanup_dir {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
