use std::path::Path;
use std::process::{Command, Stdio};

use crate::plan::TestExecutionUnit;

pub(crate) fn execute(
    units: &[TestExecutionUnit],
    target_root: Option<&Path>,
) -> Result<(), String> {
    for unit in units {
        println!("run: {}", unit.identity());
        let mut command = command(unit, target_root);
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let status = command
            .status()
            .map_err(|error| format!("failed to start `{}`: {error}", unit.display_command()))?;
        if !status.success() {
            return Err(format!("unit `{}` exited with {status}", unit.identity()));
        }
    }
    Ok(())
}

fn command(unit: &TestExecutionUnit, target_root: Option<&Path>) -> Command {
    let mut command = Command::new(unit.program());
    command.args(unit.arguments()).current_dir(unit.directory());
    if unit.program() == "cargo" {
        if let Some(root) = target_root {
            command.env("CARGO_TARGET_DIR", root);
        }
    }
    command
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
