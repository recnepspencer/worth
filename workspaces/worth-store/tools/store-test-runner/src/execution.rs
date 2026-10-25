use std::path::Path;
use std::process::{Command, Stdio};

use crate::plan::TestExecutionUnit;

pub(crate) fn execute(
    units: &[TestExecutionUnit],
    target_root: Option<&Path>,
) -> Result<(), String> {
    for unit in units {
        println!("run: {}", unit.identity());
        let mut command = command(unit, target_root)?;
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

fn command(unit: &TestExecutionUnit, target_root: Option<&Path>) -> Result<Command, String> {
    let mut command = if unit.program() == "bash" {
        // Windows searches its system directory before PATH for a bare name.
        // Honor the selected shell instead of accidentally launching WSL.
        Command::new(bash_on_path()?)
    } else {
        Command::new(unit.program())
    };
    command.args(unit.arguments()).current_dir(unit.directory());
    if unit.program() == "cargo" {
        if unit.arguments().windows(2).any(|pair| {
            matches!(pair[0].as_str(), "--profile" | "--cargo-profile") && pair[1] == "ci-test"
        }) {
            // The repository's build.incremental setting overrides profiles.
            // Preserve the CI profile's declared non-incremental compilation.
            command.env("CARGO_INCREMENTAL", "0");
        }
        if let Some(root) = target_root {
            command.env("CARGO_TARGET_DIR", root);
        }
    }
    Ok(command)
}

fn bash_on_path() -> Result<std::path::PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or("structural checks require Bash on PATH")?;
    let executable = format!("bash{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(&path)
        .map(|directory| directory.join(&executable))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| "structural checks require Bash on PATH".to_owned())
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
