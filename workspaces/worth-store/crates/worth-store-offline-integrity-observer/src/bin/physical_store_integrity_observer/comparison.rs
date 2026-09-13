use super::arguments::{ArgumentOutcome, HELP};
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use worth_store_offline_integrity_observer::{
    compare_integrity_observations, PhysicalIntegrityComparisonLimits,
};

pub(super) struct CompareArguments {
    runtime: PathBuf,
    offline: PathBuf,
    report: OsString,
}

pub(super) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<CompareArguments, ArgumentOutcome> {
    let mut runtime = None;
    let mut offline = None;
    let mut report = None;
    let mut arguments = arguments.into_iter();
    while let Some(flag) = arguments.next() {
        if flag == "--help" || flag == "-h" {
            return Err(ArgumentOutcome::Help(HELP));
        }
        let value = arguments.next().ok_or_else(|| {
            ArgumentOutcome::Denied(format!("{} requires a value", flag.to_string_lossy()))
        })?;
        let slot = match flag.to_str() {
            Some("--runtime-observation") => &mut runtime,
            Some("--offline-observation") => &mut offline,
            Some("--report") => &mut report,
            _ => {
                return Err(ArgumentOutcome::Denied(format!(
                    "unknown argument {}",
                    flag.to_string_lossy()
                )))
            }
        };
        if slot.replace(value).is_some() {
            return Err(ArgumentOutcome::Denied(format!(
                "duplicate {}",
                flag.to_string_lossy()
            )));
        }
    }
    Ok(CompareArguments {
        runtime: PathBuf::from(
            runtime
                .ok_or_else(|| ArgumentOutcome::Denied("missing --runtime-observation".into()))?,
        ),
        offline: PathBuf::from(
            offline
                .ok_or_else(|| ArgumentOutcome::Denied("missing --offline-observation".into()))?,
        ),
        report: report.ok_or_else(|| ArgumentOutcome::Denied("missing --report".into()))?,
    })
}

pub(super) fn compare(arguments: CompareArguments) -> Result<(), String> {
    let runtime = std::fs::canonicalize(&arguments.runtime)
        .map_err(|error| format!("runtime input unavailable: {error}"))?;
    let offline = std::fs::canonicalize(&arguments.offline)
        .map_err(|error| format!("offline input unavailable: {error}"))?;
    let destination = if arguments.report == "-" {
        None
    } else {
        let path = PathBuf::from(&arguments.report);
        let parent = std::fs::canonicalize(
            path.parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new(".")),
        )
        .map_err(|error| format!("report parent unavailable: {error}"))?;
        let filename = path.file_name().ok_or("report must name a file")?;
        let path = parent.join(filename);
        if path == runtime || path == offline {
            return Err("report destination equals an input report".into());
        }
        Some(path)
    };
    let limits = PhysicalIntegrityComparisonLimits::default();
    let runtime_wire = read_bounded(&runtime, limits.maximum_input_bytes())?;
    let offline_wire = read_bounded(&offline, limits.maximum_input_bytes())?;
    let comparison = compare_integrity_observations(&runtime_wire, &offline_wire, limits)
        .map_err(|denial| format!("comparison denied: {denial:?}"))?;
    if let Some(path) = destination {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("report creation failed: {error}"))?;
        file.write_all(comparison.encoded_report().as_bytes())
            .map_err(|error| format!("report emission failed: {error}"))?;
    } else {
        std::io::stdout()
            .lock()
            .write_all(comparison.encoded_report().as_bytes())
            .map_err(|error| format!("stdout emission failed: {error}"))?;
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: u64) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("input open failed: {error}"))?;
    let before = file
        .metadata()
        .map_err(|error| format!("input metadata failed: {error}"))?;
    if !before.is_file() || before.len() > maximum {
        return Err("input report bound exceeded or not a regular file".into());
    }
    let mut reader = file.take(maximum + 1);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| format!("input read failed: {error}"))?;
    let after = reader
        .get_ref()
        .metadata()
        .map_err(|error| format!("input metadata failed: {error}"))?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err("input report changed during comparison acquisition".into());
    }
    String::from_utf8(bytes).map_err(|_| "input report is not UTF-8".into())
}
