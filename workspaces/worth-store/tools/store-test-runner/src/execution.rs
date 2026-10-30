use std::path::Path;
use std::process::{Command, Stdio};

use crate::plan::TestExecutionUnit;

pub(crate) fn execute(
    units: &[TestExecutionUnit],
    target_root: Option<&Path>,
) -> Result<(), String> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("runner lives under workspaces/worth-store/tools");
    let plan = units
        .iter()
        .map(|unit| {
            (
                unit.identity(),
                unit.directory(),
                unit.program(),
                unit.arguments(),
            )
        })
        .collect::<Vec<_>>();
    let mut command =
        Command::new(std::env::var_os("WORTH_PYTHON").unwrap_or_else(|| "python".into()));
    command
        .arg(repository.join("scripts/test_cost/store_plan.py"))
        .current_dir(repository)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(root) = target_root {
        command.env("CARGO_TARGET_DIR", root);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start prebuilt Store plan: {error}"))?;
    serde_json::to_writer(child.stdin.take().expect("piped plan input"), &plan)
        .map_err(|error| format!("cannot send Store execution plan: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("cannot join Store execution plan: {error}"))?;
    status.success().then_some(()).ok_or_else(|| {
        format!(
            "Store units [{}] exited with {status}",
            units
                .iter()
                .map(TestExecutionUnit::identity)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::execute;
    use crate::plan::TestExecutionUnit;

    #[test]
    fn child_failure_is_returned_with_the_unit_identity() {
        let unit = TestExecutionUnit::command(
            "failing-child",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            "rustc",
            &["--definitely-not-a-rustc-option"],
        );

        let error = execute(&[unit], None).unwrap_err();
        assert!(error.contains("failing-child"));
        assert!(error.contains("exit"));
    }
}
