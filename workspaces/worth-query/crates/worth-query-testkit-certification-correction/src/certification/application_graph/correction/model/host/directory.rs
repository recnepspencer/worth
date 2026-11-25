//! Real Directory Home open/reopen helpers for the numeric correction model.
use super::*;
use std::path::PathBuf;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

/// Open or reopen a real Directory Home for the current numeric fixture.
pub fn open_directory(directory: PathBuf) -> DirHost {
    open_directory_with_seed_and_limits(
        directory,
        validated::<Program>(),
        roster().support(validated::<super::super::output_program::OutputProgram>()),
        |_| Ok(()),
        installation_limits(),
    )
}

/// Open a new Directory Home and retain its bootstrap projection owner.
pub fn open_directory_with_projection(
    directory: PathBuf,
) -> (
    DirHost,
    WorthQueryApplicationInvariantProjectionAuthority<NumericSchema>,
) {
    let mut authority = None;
    let home = open_directory_with_seed_and_limits(
        directory,
        validated::<Program>(),
        roster().support(validated::<super::super::output_program::OutputProgram>()),
        |graph| {
            authority = Some(graph.retain_invariant_projection_authority());
            Ok(())
        },
        installation_limits(),
    );
    (
        home,
        authority.expect("actual bootstrap issued projection authority"),
    )
}

pub fn open_directory_with_outputs(
    directory: PathBuf,
    limits: WorthQueryApplicationLimits,
) -> DirOutputHost {
    open_directory_with_seed_and_limits(
        directory,
        validated::<super::super::output_program::OutputProgram>(),
        roster().support(validated::<Program>()),
        |_| Ok(()),
        limits,
    )
}

pub(in super::super) fn open_directory_with_seed_and_limits<P>(
    directory: PathBuf,
    program: ValidatedApplicationProgram<NumericSchema, P>,
    roster: WorthQueryApplicationProgramRoster<'_, NumericSchema>,
    seed: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    limits: WorthQueryApplicationLimits,
) -> DirectoryHomeGuard<NumericSchema, P>
where
    P: ApplicationProgramDefinition<NumericSchema, Contributions = (NumericContribution,)>
        + 'static,
    P::Outputs: WorthQueryApplicationProgramRoots<NumericSchema>,
{
    let home = with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            program,
            roster,
            NumericSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            limits,
            |_, graph, installed| {
                seed_numeric_graph(graph, installed)?;
                seed(graph)
            },
        )
        .open_in_advancement(phase)
    })
    .unwrap_or_else(|refused| panic!("numeric directory Home open refused: {refused:?}"));
    DirectoryHomeGuard::wrap(home, directory)
}
