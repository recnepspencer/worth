use std::path::{Path, PathBuf};

pub(super) struct Invocation {
    pub(super) root: PathBuf,
    pub(super) output: PathBuf,
    pub(super) maximum_directory_entries: String,
    pub(super) maximum_directories: String,
    pub(super) maximum_artifacts: String,
    pub(super) maximum_bytes: String,
}

pub(super) fn parse(
    mut arguments: impl Iterator<Item = std::ffi::OsString>,
) -> Result<Invocation, String> {
    let root = required(&mut arguments);
    let output = required(&mut arguments);
    let maximum_directory_entries = required(&mut arguments);
    let maximum_directories = required(&mut arguments);
    let maximum_artifacts = required(&mut arguments);
    let maximum_bytes = required(&mut arguments);
    if arguments.next().is_some() {
        super::super::usage();
    }
    let root = PathBuf::from(root);
    let output = PathBuf::from(output);
    reject_report_inside_store(&root, &output)?;
    Ok(Invocation {
        root,
        output,
        maximum_directory_entries: maximum_directory_entries.to_string_lossy().into_owned(),
        maximum_directories: maximum_directories.to_string_lossy().into_owned(),
        maximum_artifacts: maximum_artifacts.to_string_lossy().into_owned(),
        maximum_bytes: maximum_bytes.to_string_lossy().into_owned(),
    })
}

fn required(arguments: &mut impl Iterator<Item = std::ffi::OsString>) -> std::ffi::OsString {
    arguments.next().unwrap_or_else(|| super::super::usage())
}

fn reject_report_inside_store(root: &Path, report: &Path) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve observer Store root: {error}"))?;
    let report = if report.exists() {
        report
            .canonicalize()
            .map_err(|error| format!("could not resolve observer report path: {error}"))?
    } else {
        let parent = report
            .parent()
            .ok_or_else(|| "observer report path has no parent".to_owned())?
            .canonicalize()
            .map_err(|error| format!("could not resolve observer report directory: {error}"))?;
        parent.join(
            report
                .file_name()
                .ok_or_else(|| "observer report path has no file name".to_owned())?,
        )
    };
    if report == root || report.starts_with(&root) {
        return Err("observer report must be outside the observed Store root".to_owned());
    }
    Ok(())
}
