//! Identity of the observer and its checksum-verified, adjacent compiler runtime.
use crate::returned_artifacts::digest;
use std::{
    fs,
    path::{Path, PathBuf},
};

// From the official 2026-03-05 rustc 1.94.0 archive, verified by stage.py.
const RUNTIME: &[(&str, &str)] = &[
    (
        "rustc_driver-eab2645d391441c2.dll",
        "40000cce93f49d820e54c5025e1305e7019a0ff98989fc63d1259541ffa2f6c9",
    ),
    (
        "std-2137bdd3874dafb5.dll",
        "d6613a5432ec4610f714266889a18b1bdccb6035b3a3add2019233a5eb554637",
    ),
];

pub(super) fn verified() -> Result<(String, PathBuf), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let directory = executable.parent().ok_or("missing compiler directory")?;
    let mut identity = fs::read(&executable).map_err(|e| e.to_string())?;
    for (name, expected) in RUNTIME {
        let bytes = fs::read(directory.join(name)).map_err(|e| format!("{name}: {e}"))?;
        if digest(&bytes) != *expected {
            return Err(format!("unverified pinned compiler runtime: {name}"));
        }
        identity.extend_from_slice(name.as_bytes());
        identity.extend_from_slice(expected.as_bytes());
    }
    let sysroot = directory.join("pinned-sysroot");
    let mut records = Vec::new();
    tree(&sysroot, &sysroot, &mut records)?;
    records.sort();
    let basis: String = records
        .into_iter()
        .map(|(name, hash)| format!("{name}\0{hash}\n"))
        .collect();
    let hash = digest(basis.as_bytes());
    if hash != "e88a2b78310c7033e4a0d1c54dc9fa6e8f87dd1e9397fac8dfda6a93b9e51a56" {
        return Err("unverified pinned standard-library tree".into());
    }
    identity.extend_from_slice(hash.as_bytes());
    Ok((digest(&identity), sysroot))
}

fn tree(root: &Path, directory: &Path, records: &mut Vec<(String, String)>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("linked pinned sysroot input".into());
        }
        if kind.is_dir() {
            tree(root, &entry.path(), records)?;
        } else if kind.is_file() {
            records.push((
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("non-UTF8 pinned sysroot input")?
                    .replace('\\', "/"),
                digest(&fs::read(entry.path()).map_err(|e| e.to_string())?),
            ));
        } else {
            return Err("non-regular pinned sysroot input".into());
        }
    }
    Ok(())
}
