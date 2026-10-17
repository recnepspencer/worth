use std::{fs, path::Path, process::Command};

use serde_json::{json, Value};

pub(crate) const SESSION: &str = "WORTH_IMPLEMENTATION_OBSERVER_SESSION";
pub(crate) const REOBSERVE: &str = "WORTH_IMPLEMENTATION_REOBSERVE";

pub(crate) fn build(
    manifest: &Path,
    target: &Path,
    arguments: &[String],
) -> Result<Vec<crate::ObservedCompilation>, String> {
    if std::env::var_os(SESSION).is_some() {
        return Err("nested or caller-supplied observer session is refused".into());
    }
    if arguments.iter().any(|arg| {
        arg.starts_with("--target-dir") || arg.starts_with("--manifest-path") || arg == "--config"
    }) {
        return Err("manifest/target/config overrides must not bypass the observer".into());
    }
    let manifest = fs::canonicalize(manifest).map_err(|error| error.to_string())?;
    fs::create_dir_all(target).map_err(|error| error.to_string())?;
    let target = fs::canonicalize(target).map_err(|error| error.to_string())?;
    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--format-version=1",
            "--locked",
            "--manifest-path",
        ])
        .arg(&manifest)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .map_err(|error| error.to_string())?;
    if !metadata.status.success() {
        return Err(String::from_utf8_lossy(&metadata.stderr).into_owned());
    }
    let metadata: Value =
        serde_json::from_slice(&metadata.stdout).map_err(|error| error.to_string())?;
    let mut packages = Vec::new();
    for package in metadata["packages"]
        .as_array()
        .ok_or("Cargo packages missing")?
    {
        // A registry/git name and lock entry alone do not verify its content.
        // Those source kinds will enter only with archive/tree verification.
        if !package["source"].is_null() {
            return Err(format!(
                "unqualified external package content: {}",
                package["id"]
            ));
        }
        let manifest = Path::new(
            package["manifest_path"]
                .as_str()
                .ok_or("package manifest missing")?,
        );
        let root = fs::canonicalize(manifest.parent().ok_or("package root missing")?)
            .map_err(|error| error.to_string())?;
        packages.push(json!({
            "name": package["name"], "version": package["version"],
            "root": root,
        }));
    }
    let wrapper = std::env::var_os("RUSTC_WRAPPER").unwrap_or_else(|| "sccache".into());
    if Path::new(&wrapper)
        .file_stem()
        .is_none_or(|name| name != "sccache")
    {
        return Err("configured downstream wrapper must be sccache".into());
    }
    if std::env::var_os("RUSTC_WORKSPACE_WRAPPER").is_some() {
        return Err("an unobserved nested workspace wrapper is refused".into());
    }
    let state = target.join("implementation-edition");
    fs::create_dir_all(&state).map_err(|error| error.to_string())?;
    let session = state.join("session.json");
    let run = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    );
    let compiler = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let compiler_path = crate::compiler_context::compiler_path(&compiler)?;
    let identity = crate::compiler_context::compiler_identity(
        &compiler,
        crate::compiler_context::flag(arguments, "--target"),
    )?;
    let config = json!({"packages": packages, "wrapper": wrapper.to_string_lossy(), "state": state, "run": run,
        "compiler_path": compiler_path, "compiler_identity": identity});
    fs::write(&session, serde_json::to_vec(&config).unwrap()).map_err(|error| error.to_string())?;
    let output = Command::new("cargo")
        .args([
            "build",
            "--locked",
            "-j",
            "4",
            "--message-format=json-render-diagnostics",
            "--manifest-path",
        ])
        .arg(manifest)
        .arg("--target-dir")
        .arg(target)
        .args(arguments)
        .env(
            "RUSTC_WRAPPER",
            std::env::current_exe().map_err(|error| error.to_string())?,
        )
        .env(SESSION, session)
        .env(REOBSERVE, &run)
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .map_err(|error| error.to_string())?;
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(
            "observed Cargo build failed; no component was admitted for a failed unit".into(),
        );
    }
    let mut compilations = Vec::new();
    // Descriptions from prior invocations are deliberately insufficient. Every
    // admitted component must have a compiler observation in this build.
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let message: Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
        if message["reason"] == "compiler-artifact"
            && !message["target"]["kind"]
                .as_array()
                .unwrap()
                .iter()
                .any(|kind| kind == "custom-build")
        {
            for file in message["filenames"]
                .as_array()
                .ok_or("Cargo artifact filenames missing")?
            {
                let artifact = Path::new(file.as_str().ok_or("Cargo artifact path missing")?);
                if artifact
                    .extension()
                    .is_some_and(|extension| matches!(extension.to_str(), Some("rlib" | "exe")))
                    || (!cfg!(windows) && artifact.extension().is_none())
                {
                    let observed = crate::stamp::cargo_product(artifact, &run)?;
                    compilations.push(crate::ObservedCompilation {
                        artifact: artifact.to_owned(),
                        implementation: crate::CompiledImplementation {
                            closure: observed["closure"].as_str().unwrap().into(),
                            context: observed["context"].as_str().unwrap().into(),
                        },
                    });
                }
            }
        } else if message["reason"] == "compiler-message"
            && let Some(rendered) = message["message"]["rendered"].as_str()
        {
            eprint!("{rendered}");
        }
    }
    Ok(compilations)
}
