use std::path::Path;

use super::TestExecutionUnit;

pub(super) fn owner(package: &str, workspace_root: &Path) -> Vec<TestExecutionUnit> {
    vec![TestExecutionUnit::cargo(
        format!("owner::{package}"),
        workspace_root,
        vec![
            "nextest".into(),
            "run".into(),
            "-p".into(),
            package.into(),
            "--lib".into(),
            "--bins".into(),
            "--examples".into(),
            "--benches".into(),
            "--no-fail-fast".into(),
            "--no-tests=fail".into(),
        ],
    )]
}

pub(super) fn owner_ci(workspace_root: &Path) -> Vec<TestExecutionUnit> {
    vec![
        TestExecutionUnit::cargo(
            "owner-unit::workspace-targets".into(),
            workspace_root,
            [
                "nextest",
                "run",
                "--workspace",
                "--lib",
                "--bins",
                "--examples",
                "--benches",
                "--no-fail-fast",
                "--no-tests=fail",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        ),
        TestExecutionUnit::cargo(
            "owner-unit::workspace-doctests".into(),
            workspace_root,
            ["test", "-q", "--workspace", "--doc"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        ),
    ]
}
