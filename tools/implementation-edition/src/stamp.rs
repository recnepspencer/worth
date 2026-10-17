//! Persisted descriptions are accepted only after content revalidation. They
//! cannot be deserialized into the public compiled component.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

use crate::{fingerprint, inputs, CompiledImplementation};

pub(crate) struct VerifiedUnit {
    pub(crate) implementation: CompiledImplementation,
    pub(crate) description: Value,
}

pub(crate) fn admit(
    context: Value,
    inputs: Vec<Value>,
    dependencies: Vec<Value>,
    generator: Value,
    generator_reads: Vec<Value>,
) -> VerifiedUnit {
    let context_digest = fingerprint::value("worth.compiler.context.v1", &context);
    let basis = json!({
        "context": context,
        "inputs": inputs.iter().map(|input| json!([input["identity"], input["digest"]])).collect::<Vec<_>>(),
        "dependencies": dependencies.iter().map(|dependency| dependency["closure"].clone()).collect::<Vec<_>>(),
        "generator": generator,
    });
    let closure = fingerprint::value("worth.compiler.implementation.v1", &basis);
    VerifiedUnit {
        implementation: CompiledImplementation {
            closure: closure.clone(),
            context: context_digest.clone(),
        },
        description: json!({"version": 1, "closure": closure, "context_digest": context_digest,
            "basis": basis, "inputs": inputs, "dependencies": dependencies, "generator_reads": generator_reads}),
    }
}

pub(crate) fn store(unit: &VerifiedUnit, artifacts: &[PathBuf]) -> Result<(), String> {
    for artifact in artifacts {
        let mut description = unit.description.clone();
        description["observation"] = json!(std::env::var(crate::cargo_build::REOBSERVE)
            .map_err(|_| "compiler observation run missing")?);
        description["artifact"] = json!({"path": artifact, "digest": fingerprint::bytes(&fs::read(artifact).map_err(|error| error.to_string())?)});
        let stamp = path(artifact);
        let temporary = stamp.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, serde_json::to_vec_pretty(&description).unwrap())
            .map_err(|error| error.to_string())?;
        // Each Cargo unit owns its artifacts; no competing writer uses this slot.
        if stamp.exists() {
            fs::remove_file(&stamp).map_err(|error| error.to_string())?;
        }
        fs::rename(temporary, stamp).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn dependency(artifact: &Path) -> Result<Value, String> {
    let run = std::env::var(crate::cargo_build::REOBSERVE)
        .map_err(|_| "compiler observation run missing")?;
    dependency_in(artifact, &run)
}

pub(crate) fn dependency_in(artifact: &Path, run: &str) -> Result<Value, String> {
    verify(artifact, &mut BTreeSet::new(), run)
}

pub(crate) fn is_current(artifact: &Path, run: &str) -> bool {
    fs::read(path(artifact))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some_and(|description| description["observation"] == run)
}

/// Cargo copies final named products out of deps. Accept that alias only when
/// its bytes equal exactly one current observed compiler product/component.
pub(crate) fn cargo_product(artifact: &Path, run: &str) -> Result<Value, String> {
    if is_current(artifact, run) {
        return dependency_in(artifact, run);
    }
    let directory = artifact
        .parent()
        .ok_or("Cargo product directory missing")?
        .join("deps");
    if !directory.is_dir() {
        return dependency_in(artifact, run);
    }
    let base = artifact
        .file_stem()
        .ok_or("Cargo product name missing")?
        .to_string_lossy()
        .replace('-', "_");
    let prefix = base.clone() + "-";
    let content = fs::read(artifact).map_err(|error| error.to_string())?;
    let mut matched: Option<Value> = None;
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let candidate = entry.map_err(|error| error.to_string())?.path();
        if candidate.extension() != artifact.extension()
            || !(candidate.file_stem().unwrap().to_string_lossy() == base
                || candidate
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&prefix))
            || !is_current(&candidate, run)
        {
            continue;
        }
        if fs::read(&candidate).map_err(|error| error.to_string())? != content {
            continue;
        }
        let verified = dependency_in(&candidate, run)?;
        if matched
            .as_ref()
            .is_some_and(|prior| prior["closure"] != verified["closure"])
        {
            return Err("ambiguous observed Cargo product alias".into());
        }
        let mut description: Value =
            serde_json::from_slice(&fs::read(path(&candidate)).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        description["artifact"]["path"] = json!(artifact);
        matched = Some(description);
    }
    let description = matched.ok_or_else(|| {
        format!(
            "unobserved Cargo product {}; a plain/fresh build cannot construct a component",
            artifact.display()
        )
    })?;
    fs::write(
        path(artifact),
        serde_json::to_vec_pretty(&description).unwrap(),
    )
    .map_err(|error| error.to_string())?;
    dependency_in(artifact, run)
}

fn verify(artifact: &Path, visiting: &mut BTreeSet<PathBuf>, run: &str) -> Result<Value, String> {
    let artifact = fs::canonicalize(artifact).map_err(|error| error.to_string())?;
    if !visiting.insert(artifact.clone()) {
        return Err("cyclic compiler artifact dependency".into());
    }
    let description: Value =
        serde_json::from_slice(&fs::read(path(&artifact)).map_err(|error| {
            format!(
                "unobserved extern/generator {}: {error}; rebuild it with the observer",
                artifact.display()
            )
        })?)
        .map_err(|error| error.to_string())?;
    if description["version"] != 1 {
        return Err("unsupported compiler observation version".into());
    }
    if description["observation"] != run {
        return Err("unobserved compiler unit in this invocation; prior descriptions cannot construct a component".into());
    }
    let actual = fingerprint::bytes(&fs::read(&artifact).map_err(|error| error.to_string())?);
    if description["artifact"]["digest"] != actual {
        return Err(format!(
            "compiler artifact content changed: {}",
            artifact.display()
        ));
    }
    for input in description["inputs"]
        .as_array()
        .ok_or("observed input list missing")?
    {
        let path = Path::new(input["path"].as_str().ok_or("input path missing")?);
        let content = fs::read(path).map_err(|error| error.to_string())?;
        let actual = if input["raw"].as_bool().ok_or("input role missing")? {
            fingerprint::bytes(&content)
        } else {
            inputs::content_digest(
                path,
                &content,
                input["test"].as_bool().ok_or("input context missing")?,
            )?
        };
        if input["digest"] != actual {
            return Err(format!(
                "compiler input changed without a matching observed artifact: {}",
                path.display()
            ));
        }
    }
    for dependency in description["dependencies"]
        .as_array()
        .ok_or("dependency list missing")?
    {
        let verified = verify(
            Path::new(
                dependency["artifact"]
                    .as_str()
                    .ok_or("dependency artifact missing")?,
            ),
            visiting,
            run,
        )?;
        if verified["closure"] != dependency["closure"] {
            return Err("compiler dependency basis changed".into());
        }
    }
    let basis = &description["basis"];
    let semantic_inputs = description["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| json!([input["identity"], input["digest"]]))
        .collect::<Vec<_>>();
    let semantic_dependencies = description["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|dependency| dependency["closure"].clone())
        .collect::<Vec<_>>();
    if basis["inputs"] != json!(semantic_inputs)
        || basis["dependencies"] != json!(semantic_dependencies)
    {
        return Err("compiler observation input framing changed".into());
    }
    let closure = fingerprint::value("worth.compiler.implementation.v1", basis);
    let context = fingerprint::value("worth.compiler.context.v1", &basis["context"]);
    let generator_reads = description["generator_reads"]
        .as_array()
        .ok_or("generator read contract missing")?;
    if basis["context"]["generator_reads"]
        != json!(generator_reads
            .iter()
            .map(|input| input["identity"].clone())
            .collect::<Vec<_>>())
    {
        return Err("observed generator read contract differs from compiler context".into());
    }
    if description["closure"] != closure || description["context_digest"] != context {
        return Err("compiler observation fingerprint does not match its inputs/context".into());
    }
    visiting.remove(&artifact);
    Ok(
        json!({"artifact": artifact, "closure": closure, "context": context, "generator_reads": generator_reads}),
    )
}

pub(crate) fn path(artifact: &Path) -> PathBuf {
    PathBuf::from(format!("{}.implementation.json", artifact.display()))
}
