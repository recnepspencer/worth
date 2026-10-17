//! A compiler-produced dep-info trailer binds audit to its returned artifact bytes.
//! It is evidence only; no SDK authority or component can be deserialized from it.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf};

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn bind(files: &[PathBuf], expanded: &str, compiler: &str) -> Result<(), String> {
    let Some(dep_info) = files
        .iter()
        .find(|path| path.extension().is_some_and(|ext| ext == "d"))
    else {
        return Err("audited compilation requires returned dep-info".into());
    };
    let mut artifacts = Vec::new();
    for path in files.iter().filter(|path| *path != dep_info) {
        if path.is_file() {
            artifacts.push(json!({"name":path.file_name().unwrap().to_string_lossy(),"sha256":digest(&fs::read(path).map_err(|e| e.to_string())?)}));
        }
    }
    // sccache invokes a dep-info-only discovery. It has no returned artifact
    // and consequently receives no audit/artifact evidence.
    if artifacts.is_empty() {
        return Ok(());
    }
    let invocation = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    let evidence = json!({"version":1,"driver":compiler,"expanded":expanded,"invocation":invocation,"artifacts":artifacts});
    writeln!(
        fs::OpenOptions::new()
            .append(true)
            .open(dep_info)
            .map_err(|e| e.to_string())?,
        "\n# worth-expanded-audit={evidence}"
    )
    .map_err(|e| e.to_string())
}
