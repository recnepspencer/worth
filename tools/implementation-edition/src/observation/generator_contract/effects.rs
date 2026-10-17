use super::{evaluation::Evaluation, values::Fault, Program};
use serde_json::Value as Json;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub(in crate::observation) struct Effects {
    pub(in crate::observation) stdout: String,
    pub(in crate::observation) writes: BTreeMap<PathBuf, Vec<u8>>,
    pub(in crate::observation) reads: BTreeMap<PathBuf, Vec<u8>>,
    pub(in crate::observation) environment: BTreeMap<String, Json>,
    pub(in crate::observation) queries: Vec<Json>,
}

impl Program {
    pub(in crate::observation) fn evaluate(
        &self,
        root: &Path,
        output: &Path,
        compiler: &Path,
    ) -> Result<Effects, String> {
        let environment = std::env::vars_os()
            .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
            .collect();
        self.evaluate_with_environment(root, output, compiler, &environment)
    }

    pub(super) fn evaluate_with_environment(
        &self,
        root: &Path,
        output: &Path,
        compiler: &Path,
        environment: &BTreeMap<String, std::ffi::OsString>,
    ) -> Result<Effects, String> {
        let mut evaluation = Evaluation {
            program: self,
            frames: Vec::new(),
            effects: Effects {
                stdout: String::new(),
                writes: BTreeMap::new(),
                reads: BTreeMap::new(),
                environment: BTreeMap::new(),
                queries: Vec::new(),
            },
            root,
            output,
            compiler,
            environment,
            steps: 0,
        };
        for name in &self.environment {
            evaluation.environment_value(name).map_err(|e| match e {
                Fault::Error(e) => e,
                Fault::Return(_) => "generator environment control error".into(),
            })?;
        }
        for relative in &self.reads {
            let path = fs::canonicalize(root.join(relative)).map_err(|e| e.to_string())?;
            if !path.starts_with(root) {
                return Err("generator read escapes package".into());
            }
            evaluation
                .effects
                .reads
                .insert(path.clone(), fs::read(path).map_err(|e| e.to_string())?);
        }
        evaluation.function("main").map_err(|e| match e {
            Fault::Error(error) => error,
            Fault::Return(_) => "unhandled generator return".into(),
        })?;
        let mut declared = std::collections::BTreeSet::new();
        for line in evaluation.effects.stdout.lines() {
            let directive = line
                .strip_prefix("cargo::")
                .or_else(|| line.strip_prefix("cargo:"))
                .ok_or("generator stdout is not a Cargo directive")?;
            if let Some(path) = directive.strip_prefix("rerun-if-changed=") {
                declared.insert(fs::canonicalize(root.join(path)).map_err(|e| e.to_string())?);
            } else if !directive.starts_with("rustc-cfg=")
                && !directive.starts_with("rustc-check-cfg=")
                && !directive.starts_with("warning=")
            {
                return Err("generator directive requires effect qualification".into());
            }
        }
        for path in &self.reads {
            let path = fs::canonicalize(root.join(path)).map_err(|e| e.to_string())?;
            if !declared.contains(&path) {
                return Err(format!(
                    "generator emitted directives omit validated file read: {}",
                    path.display()
                ));
            }
        }
        Ok(evaluation.effects)
    }
}
