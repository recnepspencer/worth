use std::path::Path;

use super::TestExecutionUnit;

pub(super) fn process_scenario(workspace_root: &Path) -> Vec<TestExecutionUnit> {
    vec![TestExecutionUnit::cargo(
        "process-scenario::fresh-process-recovery".into(),
        workspace_root,
        vec![
            "run".into(),
            "--locked".into(),
            "-q".into(),
            "-p".into(),
            "store-test-runner".into(),
            "--bin".into(),
            "store_process_scenario".into(),
        ],
    )]
}
