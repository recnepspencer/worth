use flate2::read::GzDecoder;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(super) fn files(package: &Value) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    let checksum = package["checksum"]
        .as_str()
        .ok_or("locked archive checksum missing")?;
    let filename = format!(
        "{}-{}.crate",
        package["name"].as_str().unwrap(),
        package["version"].as_str().unwrap()
    );
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|p| PathBuf::from(p).join(".cargo"))
        })
        .ok_or("Cargo home unavailable")?;
    let mut archive = None;
    for directory in fs::read_dir(home.join("registry/cache")).map_err(|e| e.to_string())? {
        let path = directory.map_err(|e| e.to_string())?.path().join(&filename);
        if path.is_file() {
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            if crate::fingerprint::bytes(&bytes) == checksum {
                archive = Some(bytes);
                break;
            }
        }
    }
    let bytes = archive.ok_or("no registry archive matches the locked checksum")?;
    let prefix = format!(
        "{}-{}",
        package["name"].as_str().unwrap(),
        package["version"].as_str().unwrap()
    );
    let mut result = BTreeMap::new();
    for entry in tar::Archive::new(GzDecoder::new(bytes.as_slice()))
        .entries()
        .map_err(|e| e.to_string())?
    {
        let mut entry = entry.map_err(|e| e.to_string())?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err("linked/non-file archive input requires qualification".into());
        }
        let name = entry.path().map_err(|e| e.to_string())?.into_owned();
        let relative = name
            .strip_prefix(Path::new(&prefix))
            .map_err(|_| "archive package prefix differs")?
            .to_owned();
        if relative.as_os_str().is_empty()
            || !relative
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
        {
            return Err("invalid registry archive member path".into());
        }
        let mut content = Vec::new();
        entry.read_to_end(&mut content).map_err(|e| e.to_string())?;
        if result.insert(relative, content).is_some() {
            return Err("duplicate registry archive member".into());
        }
    }
    if result.is_empty() {
        return Err("registry archive has no files".into());
    }
    Ok(result)
}
