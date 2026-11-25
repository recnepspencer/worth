//! Real Directory Home open and reopen for the cell-history application.
use super::{
    host::{self, Program},
    schema::Schema,
};
use crate::facade::directory_home::{
    unique_directory, with_directory_bootstrap, DirectoryHomeGuard,
};
use std::path::PathBuf;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_installation::*, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeRequest,
};

pub fn new_directory() -> PathBuf {
    unique_directory("cell-history")
}

pub fn open_directory(directory: PathBuf, retained_steps: u64) -> DirectoryHomeGuard<Schema, Program> {
    open_directory_cells(
        directory,
        &[(10, 1, 7), (11, 1, 13), (20, 2, 29)],
        retained_steps,
    )
}

pub fn open_directory_cells(
    directory: PathBuf,
    cells: &[(u64, u64, u64)],
    retained_steps: u64,
) -> DirectoryHomeGuard<Schema, Program> {
    let home = with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            ApplicationProgramAuthoring::<Schema, Program>::begin()
                .validated_program()
                .unwrap(),
            WorthQueryApplicationProgramRoster::new(),
            Schema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("16161616-1616-1616-1616-161616161616").unwrap(),
            host::limits(retained_steps),
            |_, graph, installed| host::seed_cells(graph, installed, cells),
        )
        .open_in_advancement(phase)
    })
    .unwrap_or_else(|refused| panic!("cell_history directory open refused: {refused:?}"));
    DirectoryHomeGuard::wrap(home, directory)
}
