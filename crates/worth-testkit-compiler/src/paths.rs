use std::path::{Path, PathBuf};

use crate::CompilerFixtureRefusal;

pub(crate) fn read(path: &Path) -> Result<String, CompilerFixtureRefusal> {
    std::fs::read_to_string(path).map_err(|error| CompilerFixtureRefusal::Read {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

fn portable_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if let Some(unc) = value.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        value.strip_prefix("//?/").unwrap_or(&value).to_owned()
    }
}

pub(crate) fn source_key(path: &Path) -> String {
    let value = portable_path(path);
    if cfg!(windows) {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

pub(crate) fn expand(path: &Path) -> Result<Vec<PathBuf>, CompilerFixtureRefusal> {
    if path.file_name().is_none() {
        return Err(CompilerFixtureRefusal::EmptyPattern(path.to_path_buf()));
    }
    if !portable_path(path).contains(['*', '?', '[']) {
        return Ok(vec![path.to_path_buf()]);
    }
    let entries = glob::glob(&portable_path(path)).map_err(|error| {
        CompilerFixtureRefusal::InvalidPattern {
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| CompilerFixtureRefusal::Read {
            path: error.path().to_path_buf(),
            reason: error.to_string(),
        })?;
        let metadata = std::fs::metadata(&entry).map_err(|error| CompilerFixtureRefusal::Read {
            path: entry.clone(),
            reason: error.to_string(),
        })?;
        if metadata.is_file() {
            paths.push(entry);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(CompilerFixtureRefusal::EmptyPattern(path.to_path_buf()));
    }
    Ok(paths)
}
