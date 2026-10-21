//! Consumption of the actual, checked Cargo generator execution.
use super::generator_execution::RECORD;
use crate::{compiler_context::Context, fingerprint, stamp};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub(super) struct Generated {
    pub(super) semantic: Value,
    pub(super) dependencies: Vec<Value>,
    pub(super) inputs: Vec<Value>,
    pub(super) writes: BTreeSet<PathBuf>,
}

pub(super) fn basis(context: &Context, session: &Value) -> Result<Generated, String> {
    let Some(out) = std::env::var_os("OUT_DIR") else {
        return Ok(Generated {
            semantic: Value::Null,
            dependencies: Vec::new(),
            inputs: Vec::new(),
            writes: BTreeSet::new(),
        });
    };
    let out = fs::canonicalize(out).map_err(|e| e.to_string())?;
    let record: Value = serde_json::from_slice(
        &fs::read(out.join(RECORD)).map_err(|e| format!("unobserved generator execution: {e}"))?,
    )
    .map_err(|e| e.to_string())?;
    if record["run"] != session["run"]
        || record["out"].as_str().is_none_or(|p| Path::new(p) != out)
        || record["root"]
            .as_str()
            .is_none_or(|p| Path::new(p) != context.root)
        || record["target"] != context.semantic["compiler"]["target"]
    {
        return Err("generator execution and compiler unit context differ".into());
    }
    let dependency = stamp::dependency(Path::new(
        record["generator"]["artifact"]
            .as_str()
            .ok_or("generator artifact missing")?,
    ))?;
    if dependency != record["generator"] {
        return Err("generator component differs from execution".into());
    }
    let cargo_output = fs::read_to_string(
        out.parent()
            .ok_or("Cargo output parent missing")?
            .join("output"),
    )
    .map_err(|e| e.to_string())?;
    if record["directives"] != cargo_output {
        return Err("Cargo generator directives differ from validated execution".into());
    }
    let mut inputs = Vec::new();
    let mut writes = BTreeSet::new();
    for key in ["reads", "writes"] {
        for input in record[key]
            .as_array()
            .ok_or("generator effect records missing")?
        {
            let path = Path::new(
                input["path"]
                    .as_str()
                    .ok_or("generator effect path missing")?,
            );
            if input["digest"] != fingerprint::bytes(&fs::read(path).map_err(|e| e.to_string())?) {
                return Err("generator effect content changed".into());
            }
            if key == "writes" {
                if !path.starts_with(&out) {
                    return Err("generated write escapes output".into());
                }
                writes.insert(path.to_owned());
            }
            inputs.push(input.clone());
        }
    }
    let semantic = json!({"generator": dependency["closure"], "environment":record["environment"], "queries":record["queries"], "directives":record["directives"], "reads":record["reads"].as_array().unwrap().iter().map(|i|json!([i["identity"],i["digest"]])).collect::<Vec<_>>(), "writes":record["writes"].as_array().unwrap().iter().map(|i|json!([i["identity"],i["digest"]])).collect::<Vec<_>>()});
    Ok(Generated {
        semantic,
        dependencies: vec![dependency],
        inputs,
        writes,
    })
}
