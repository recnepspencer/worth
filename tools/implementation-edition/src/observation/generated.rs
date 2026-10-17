//! A Cargo generated-input closure includes its compiled generator, declared
//! generator file inputs and actual generated Rust. Cargo metadata alone
//! supplies none of those content observations.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::{json, Value};

use crate::{
    compiler_context::{self, Context},
    fingerprint, stamp,
};

pub(super) struct Generated {
    pub(super) semantic: Value,
    pub(super) dependencies: Vec<Value>,
    pub(super) inputs: Vec<Value>,
}

pub(super) fn basis(context: &Context, session: &Value) -> Result<Generated, String> {
    let Some(out) = std::env::var_os("OUT_DIR") else {
        return Ok(Generated {
            semantic: Value::Null,
            dependencies: Vec::new(),
            inputs: Vec::new(),
        });
    };
    let out = Path::new(&out);
    let directory = out
        .parent()
        .ok_or("generated directory has no Cargo owner")?;
    let output = fs::read_to_string(directory.join("output"))
        .map_err(|error| format!("generator output: {error}"))?;
    let build = directory.parent().ok_or("Cargo build directory missing")?;
    let prefix = format!("{}-", context.package);
    let run = std::env::var(crate::cargo_build::REOBSERVE)
        .map_err(|_| "generator observation run missing")?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(build).map_err(|error| error.to_string())? {
        let directory = entry.map_err(|error| error.to_string())?.path();
        if directory
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(&prefix)
        {
            for file in fs::read_dir(&directory).map_err(|error| error.to_string())? {
                let generator = file.map_err(|error| error.to_string())?.path();
                let name = generator.file_name().unwrap().to_string_lossy();
                if name.starts_with("build_script_build-")
                    && (generator
                        .extension()
                        .is_some_and(|extension| extension == "exe")
                        || (!cfg!(windows) && generator.extension().is_none()))
                    && stamp::is_current(&generator, &run)
                {
                    candidates.push(stamp::dependency(&generator)?);
                }
            }
        }
    }
    if candidates.len() != 1 {
        return Err("generated inputs require one unambiguous observed Cargo generator".into());
    }
    let mut files = Vec::new();
    let mut observed_files = Vec::new();
    let mut environment = Vec::new();
    let mut semantic_output = Vec::new();
    let mut emitted = BTreeSet::new();
    for line in output.lines() {
        let directive = line
            .strip_prefix("cargo::")
            .or_else(|| line.strip_prefix("cargo:"));
        let Some(directive) = directive else {
            continue;
        };
        if let Some(path) = directive.strip_prefix("rerun-if-changed=") {
            let path =
                fs::canonicalize(context.root.join(path)).map_err(|error| error.to_string())?;
            if !path.is_file() {
                return Err("generator directory/glob inputs require qualification".into());
            }
            let content = fs::read(&path).map_err(|error| error.to_string())?;
            let identity = compiler_context::normalize_path(&path.to_string_lossy(), session);
            // Runtime generator reads consume bytes, even for .rs inputs.
            let digest = fingerprint::bytes(&content);
            files.push(json!([identity, digest]));
            observed_files.push(json!({"path": path, "identity": identity, "digest": digest, "test": false, "raw": true}));
            emitted.insert(path);
        } else if let Some(name) = directive.strip_prefix("rerun-if-env-changed=") {
            environment.push(json!([
                name,
                fingerprint::bytes(std::env::var(name).unwrap_or_default().as_bytes())
            ]));
        } else if directive.starts_with("rustc-link-") || directive.starts_with("rustc-flags=") {
            return Err("native generator/linker inputs require qualification".into());
        } else if !directive.starts_with("warning=") {
            semantic_output.push(directive.to_owned());
        }
    }
    for read in candidates[0]["generator_reads"]
        .as_array()
        .ok_or("validated generator read set missing")?
    {
        let path = Path::new(
            read["path"]
                .as_str()
                .ok_or("validated generator read path missing")?,
        );
        if !emitted.contains(path) {
            return Err(format!(
                "generator emitted directives omit validated file read: {}",
                path.display()
            ));
        }
    }
    if files.is_empty() {
        return Err(
            "generator must declare finite file inputs; package-mtime fallback is refused".into(),
        );
    }
    Ok(Generated {
        semantic: json!({"generator": candidates[0]["closure"], "files": files, "environment": environment, "directives": semantic_output}),
        dependencies: candidates,
        inputs: observed_files,
    })
}
