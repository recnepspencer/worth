//! Cargo's execution boundary supplies the real generator environment. A
//! descriptive launcher invokes the original observed Rust binary unchanged.
use super::generator_contract::Program;
use crate::{
    cargo_build::{REOBSERVE, SESSION},
    compiler_context, fingerprint, stamp,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) const RECORD: &str = ".implementation-generator.json";

pub(super) fn install(
    unit: &stamp::VerifiedUnit,
    artifacts: &[PathBuf],
    source: &Path,
    root: &Path,
    session: &Value,
) -> Result<(), String> {
    if artifacts.len() != 1 {
        return Err("generator requires one native executable artifact".into());
    }
    let artifact = &artifacts[0];
    let backing = artifact.with_extension(if cfg!(windows) {
        "generator.exe"
    } else {
        "generator"
    });
    fs::copy(artifact, &backing).map_err(|e| e.to_string())?;
    stamp::store(unit, std::slice::from_ref(&backing))?;
    let dispatcher = Path::new(
        session["observer"]
            .as_str()
            .ok_or("observer executable missing")?,
    );
    fs::copy(dispatcher, artifact).map_err(|e| e.to_string())?;
    let description = json!({"run": session["run"], "backing": backing, "source": source, "root": root, "dispatcher": fingerprint::bytes(&fs::read(dispatcher).map_err(|e| e.to_string())?)});
    fs::write(
        artifact.with_extension("generator.json"),
        serde_json::to_vec(&description).unwrap(),
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn run() -> Result<(), String> {
    let session: Value = serde_json::from_slice(
        &fs::read(
            std::env::var_os(SESSION).ok_or("generator observation requires its Cargo session")?,
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let run = std::env::var(REOBSERVE).map_err(|_| "generator observation run missing")?;
    if session["run"] != run {
        return Err("generator observation run differs".into());
    }
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(current.parent().ok_or("generator directory missing")?)
        .map_err(|e| e.to_string())?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".generator.json")
        {
            let description: Value =
                serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if description["run"] == run {
                candidates.push(description);
            }
        }
    }
    if candidates.len() != 1 {
        return Err("generator requires one current dispatcher description".into());
    }
    let description = &candidates[0];
    if description["dispatcher"]
        != fingerprint::bytes(&fs::read(&current).map_err(|e| e.to_string())?)
    {
        return Err("generator dispatcher content differs".into());
    }
    let backing = Path::new(
        description["backing"]
            .as_str()
            .ok_or("generator backing missing")?,
    );
    let dependency = stamp::dependency(backing)?;
    let source = fs::canonicalize(
        description["source"]
            .as_str()
            .ok_or("generator source missing")?,
    )
    .map_err(|e| e.to_string())?;
    let compiled: Value =
        serde_json::from_slice(&fs::read(stamp::path(backing)).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if !compiled["inputs"].as_array().unwrap().iter().any(|input| {
        input["raw"] == false
            && input["path"]
                .as_str()
                .is_some_and(|p| Path::new(p) == source)
    }) {
        return Err("generator contract is not its compiled source".into());
    }
    let root = fs::canonicalize(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or("generator Cargo root missing")?,
    )
    .map_err(|e| e.to_string())?;
    if description["root"]
        .as_str()
        .is_none_or(|p| Path::new(p) != root)
    {
        return Err("generator execution root differs".into());
    }
    let out =
        fs::canonicalize(std::env::var_os("OUT_DIR").ok_or("generator output directory missing")?)
            .map_err(|e| e.to_string())?;
    if fs::canonicalize(std::env::current_dir().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?
        != root
    {
        return Err("generator working directory differs from Cargo package".into());
    }
    let compiler = fs::canonicalize(
        session["compiler_path"]
            .as_str()
            .ok_or("selected compiler missing")?,
    )
    .map_err(|e| e.to_string())?;
    let program = Program::parse(&fs::read_to_string(source).map_err(|e| e.to_string())?)?;
    let effects = program.evaluate(&root, &out, &compiler)?;
    let package = session["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| {
            package["root"]
                .as_str()
                .is_some_and(|p| Path::new(p) == root)
        })
        .ok_or("generator package missing")?;
    let content = crate::package_content::verify(package, &session)?;
    for path in effects.reads.keys() {
        if !crate::package_content::contains_input(&content, &root, path) {
            return Err("unverified external generator file read".into());
        }
    }
    let record_path = out.join(RECORD);
    if record_path.exists() {
        fs::remove_file(&record_path).map_err(|e| e.to_string())?;
    }
    let output = Command::new(backing).output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "native generator failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != effects.stdout.as_bytes() {
        return Err("native generator directives differ from validated effects".into());
    }
    let mut writes = Vec::new();
    for (path, expected) in &effects.writes {
        let actual = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if !actual.starts_with(&out) || fs::read(&actual).map_err(|e| e.to_string())? != *expected {
            return Err("native generator write differs from validated effect".into());
        }
        writes.push(json!({"path": actual, "identity": compiler_context::normalize_path(&actual.to_string_lossy(), &session), "digest": fingerprint::bytes(expected), "raw":true,"test":false}));
    }
    let mut reads = Vec::new();
    for (path, expected) in &effects.reads {
        if fs::read(path).map_err(|e| e.to_string())? != *expected {
            return Err("generator read changed during execution".into());
        }
        reads.push(json!({"path":path,"identity":compiler_context::normalize_path(&path.to_string_lossy(), &session),"digest":fingerprint::bytes(expected),"raw":true,"test":false}));
    }
    stamp::dependency(backing)?;
    let record = json!({"run":run,"root":root,"out":out,"generator":dependency,"environment":effects.environment,"queries":effects.queries,"directives":effects.stdout,"writes":writes,"reads":reads,"target":std::env::var("TARGET").map_err(|_| "generator TARGET missing")?});
    fs::write(record_path, serde_json::to_vec_pretty(&record).unwrap())
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}
