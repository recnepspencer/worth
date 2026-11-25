//! Real Directory Home open and reopen helpers for owned-output courts.
use super::*;
use std::path::PathBuf;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::directory_home::{
    directory_lease, unique_directory, DirectoryHomeGuard,
};

pub type DirHost<P = program::Program> = DirectoryHomeGuard<Schema, P>;

pub fn open_directory_host(worker: bool) -> DirHost {
    let limits = if worker {
        worker_limits()
    } else {
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        )
    };
    open_directory_program::<program::Program>(limits)
}

pub fn open_directory_program<P>(
    limits: WorthQueryApplicationLimits,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    open_directory_program_with_value::<P>(limits, 1)
}

pub fn open_directory_program_with_value<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    open_directory_program_with_other_model::<P>(limits, initial_value, None)
}

pub fn open_directory_program_with_other_model<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
    other_value: Option<u64>,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    open_directory_configured::<P>(limits, initial_value, other_value, Configuration::default())
}

pub fn open_directory_configured<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
    other_value: Option<u64>,
    configuration: Configuration,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    open_directory_at::<P>(
        unique_directory("owned-outputs"),
        limits,
        initial_value,
        other_value,
        configuration,
        false,
    )
}

pub fn reopen_same_build(directory: PathBuf, worker: bool) -> DirHost {
    reopen_program_directory::<program::Program>(directory, worker, false)
}

pub fn reopen_program_directory<P>(
    directory: PathBuf,
    worker: bool,
    changed: bool,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let limits = if worker {
        worker_limits()
    } else {
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        )
    };
    let configuration = Configuration {
        second_derivation: if changed {
            Some(|value| value + 2000)
        } else {
            None
        },
        ..Default::default()
    };
    open_directory_at::<P>(directory, limits, 1, None, configuration, changed)
}

fn open_directory_at<P>(
    directory: PathBuf,
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
    other_value: Option<u64>,
    configuration: Configuration,
    changed: bool,
) -> DirHost<P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, P>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let build = if changed {
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005357").unwrap()
    } else {
        build_identity(&configuration)
    };
    let lease = directory_lease();
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(directory.clone()),
                program,
                WorthQueryApplicationProgramRoster::new(),
                schema,
                (configuration,),
                build,
                limits,
                |_, graph, installed| {
                    seed_configured(graph, installed, initial_value, other_value, configuration)
                },
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("owned_outputs directory bootstrap request")
    .unwrap_or_else(|refused| panic!("owned_outputs directory open refused: {refused:?}"));
    DirectoryHomeGuard::wrap(home, directory)
}
