use std::path::Path;

use super::TestExecutionUnit;

pub(super) fn offline_observer_build(workspace_root: &Path) -> TestExecutionUnit {
    TestExecutionUnit::cargo(
        "00-prerequisite::physical-store-offline-observer".into(),
        workspace_root,
        [
            "build",
            "-p",
            "worth-store-offline-verifier",
            "--bin",
            "physical_store_offline_observer",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    )
}
