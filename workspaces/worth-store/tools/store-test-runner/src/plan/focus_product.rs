use std::path::Path;

use crate::product::FocusGroup;

use super::{FocusSelection, TestExecutionUnit};

pub(super) fn focus(group: FocusGroup, workspace_root: &Path) -> Vec<TestExecutionUnit> {
    let selection = FocusSelection::for_group(group);
    let arguments = if selection.process_binaries {
        [
            "run",
            "--manifest-path",
            "Cargo.toml",
            "--locked",
            "-q",
            "-p",
            "store-test-runner",
            "--bin",
            "store_process_scenario",
            "--",
            "--group",
            group.as_str(),
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    } else {
        selection.nextest_arguments()
    };
    vec![TestExecutionUnit::cargo(
        format!("focus::{}", group.as_str()),
        workspace_root,
        arguments,
    )]
}
