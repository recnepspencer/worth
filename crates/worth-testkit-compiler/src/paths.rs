use std::path::{Path, PathBuf};

use crate::CompilerFixtureRefusal;

pub(crate) fn read(path: &Path) -> Result<String, CompilerFixtureRefusal> {
    std::fs::read_to_string(path).map_err(|error| CompilerFixtureRefusal::Read {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

pub(crate) fn source_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    let value = if let Some(unc) = value.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        value.strip_prefix("//?/").unwrap_or(&value).to_owned()
    };
    if cfg!(windows) {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

pub(crate) fn expand(path: &Path) -> Result<Vec<PathBuf>, CompilerFixtureRefusal> {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(CompilerFixtureRefusal::EmptyPattern(path.to_path_buf()));
    };
    let directory = path.parent().unwrap_or(Path::new("."));
    if source_key(directory)
        .chars()
        .any(|value| matches!(value, '*' | '?' | '['))
        || name.chars().any(|value| matches!(value, '?' | '['))
        || name.matches('*').count() > 1
    {
        return Err(CompilerFixtureRefusal::UnsupportedPattern(
            path.to_path_buf(),
        ));
    }
    let Some((prefix, suffix)) = name.split_once('*') else {
        return Ok(vec![path.to_path_buf()]);
    };
    let entries = std::fs::read_dir(directory).map_err(|error| CompilerFixtureRefusal::Read {
        path: directory.to_path_buf(),
        reason: error.to_string(),
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| CompilerFixtureRefusal::Read {
            path: directory.to_path_buf(),
            reason: error.to_string(),
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(prefix) && name.ends_with(suffix) {
            let metadata = entry
                .metadata()
                .map_err(|error| CompilerFixtureRefusal::Read {
                    path: entry.path(),
                    reason: error.to_string(),
                })?;
            if metadata.is_file() {
                paths.push(entry.path());
            }
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(CompilerFixtureRefusal::EmptyPattern(path.to_path_buf()));
    }
    Ok(paths)
}
