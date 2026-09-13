use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TestExecutionUnit {
    identity: String,
    directory: PathBuf,
    program: String,
    arguments: Vec<String>,
}

impl TestExecutionUnit {
    pub(super) fn cargo(identity: String, workspace_root: &Path, arguments: Vec<String>) -> Self {
        Self {
            identity,
            directory: workspace_root.to_path_buf(),
            program: "cargo".into(),
            arguments,
        }
    }

    pub(crate) fn command(
        identity: &str,
        repository_root: PathBuf,
        program: &str,
        arguments: &[&str],
    ) -> Self {
        Self {
            identity: identity.into(),
            directory: repository_root,
            program: program.into(),
            arguments: arguments.iter().map(|value| (*value).into()).collect(),
        }
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn program(&self) -> &str {
        &self.program
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub(crate) fn display_command(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.arguments.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(super) fn apply_ci_profiles(units: &mut [TestExecutionUnit]) {
    for unit in units {
        if unit.program != "cargo" {
            continue;
        }
        if unit
            .arguments
            .starts_with(&["nextest".into(), "run".into()])
        {
            unit.arguments.splice(
                2..2,
                [
                    "--profile".into(),
                    "ci".into(),
                    "--cargo-profile".into(),
                    "ci-test".into(),
                ],
            );
        } else if matches!(
            unit.arguments.first().map(String::as_str),
            Some("test" | "build")
        ) {
            unit.arguments
                .splice(1..1, ["--profile".into(), "ci-test".into()]);
        }
    }
}
