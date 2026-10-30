//! The only effect boundary: Git enumeration, shared reads and Cargo metadata.
use crate::config::{string, table, Files, Metadata, CONFIG};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
    thread,
};

pub struct Inputs {
    pub files: Files,
    pub paths: BTreeSet<String>,
    pub manifests: BTreeMap<String, toml::Value>,
    pub graphs: BTreeMap<String, Metadata>,
    pub config: toml::Value,
}
fn command(root: &Path, program: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    if program == "cargo" {
        // Metadata performs compiler queries, not compilation to cache.
        command.env("RUSTC_WRAPPER", "");
    }
    let result = command.output().map_err(|e| e.to_string())?;
    if result.status.success() {
        Ok(result.stdout)
    } else {
        Err(format!(
            "{program} {args:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}
impl Inputs {
    pub fn read(root: &Path, config: &str) -> Result<Self, String> {
        let mut paths = BTreeSet::new();
        for args in [
            vec!["ls-files", "-z"],
            vec!["ls-files", "-z", "-o", "--exclude-standard"],
        ] {
            let output = command(root, "git", &args)?;
            for path in output.split(|b| *b == 0).filter(|p| !p.is_empty()) {
                let path = String::from_utf8(path.to_vec()).map_err(|e| e.to_string())?;
                paths.insert(path);
            }
        }
        for path in command(root, "git", &["ls-files", "--deleted", "-z"])?
            .split(|b| *b == 0)
            .filter(|p| !p.is_empty())
        {
            paths.remove(std::str::from_utf8(path).map_err(|e| e.to_string())?);
        }
        let needed: Vec<_> = paths
            .iter()
            .filter(|p| {
                p.ends_with(".rs")
                    || p.ends_with(".md")
                    || p.ends_with(".toml")
                    || p.starts_with(CONFIG)
            })
            .collect();
        let width = thread::available_parallelism().map_or(1, usize::from);
        let files = thread::scope(|s| {
            let handles: Vec<_> = needed
                .chunks(needed.len().div_ceil(width).max(1))
                .map(|chunk| {
                    s.spawn(move || {
                        chunk
                            .iter()
                            .map(|p| {
                                fs::read_to_string(root.join(p))
                                    .map(|text| ((***p).to_owned(), text.replace("\r\n", "\n")))
                                    .map_err(|e| format!("{p}: {e}"))
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("file reader panicked"))
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().flatten().collect::<Files>())
        })?;
        let config: toml::Value = toml::from_str(
            files
                .get(config)
                .ok_or("configuration is not in Git's file list")?,
        )
        .map_err(|e| e.to_string())?;
        crate::config::validate_policy(&config)?;
        for name in [
            "runtime-roots.toml",
            "court-frontier-splits.toml",
            "consumer-docs.toml",
            "control-callers.txt",
            "certification-construction-callers.txt",
            "test-workspaces.txt",
            "src-court-baseline.json",
        ] {
            let path = format!("{CONFIG}{name}");
            let text = files
                .get(&path)
                .ok_or_else(|| format!("required configuration missing: {path}"))?;
            if name.ends_with(".toml") {
                let value =
                    toml::from_str::<toml::Value>(text).map_err(|e| format!("{path}: {e}"))?;
                crate::config::validate_auxiliary(name, &value)?;
            }
            if name == "test-workspaces.txt"
                && text
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .any(|l| {
                        l.split_once('|')
                            .is_none_or(|(a, b)| a.trim().is_empty() || b.trim().is_empty())
                    })
            {
                return Err("invalid test workspace registration".into());
            }
        }
        let manifests: BTreeMap<_, _> = files
            .iter()
            .filter(|(p, _)| p.ends_with("/Cargo.toml") || *p == "Cargo.toml")
            .map(|(p, t)| {
                toml::from_str::<toml::Value>(t)
                    .map(|v| (p.clone(), v))
                    .map_err(|e| format!("{p}: {e}"))
            })
            .collect::<Result<_, _>>()?;
        let mut workspaces = BTreeSet::from(["Cargo.toml".to_owned()]);
        for key in ["subworkspaces", "context_workspaces"] {
            workspaces.extend(
                table(&config, key)
                    .iter()
                    .map(|v| format!("{}/Cargo.toml", string(v, "path"))),
            );
        }
        workspaces.extend(
            files[&format!("{CONFIG}test-workspaces.txt")]
                .lines()
                .filter(|l| !l.trim_start().starts_with('#'))
                .filter_map(|l| l.split_once('|'))
                .map(|(p, _)| format!("{}/Cargo.toml", p.trim())),
        );
        for key in [
            "dependency_denials",
            "source_dependency_allowlists",
            "dependency_target_allowlists",
        ] {
            workspaces.extend(
                table(&config, key)
                    .iter()
                    .map(|v| string(v, "workspace_manifest").into()),
            );
        }
        // A metadata session per real workspace, never per rule or package.
        for manifest in &workspaces {
            if !manifests.contains_key(manifest) {
                return Err(format!(
                    "configured workspace is outside the Git input set: {manifest}"
                ));
            }
        }
        let graphs = thread::scope(|s| {
            let handles: Vec<_> = workspaces
                .iter()
                .map(|manifest| {
                    s.spawn(move || {
                        let output = command(
                            root,
                            "cargo",
                            &[
                                "metadata",
                                "--offline",
                                "--format-version",
                                "1",
                                "--manifest-path",
                                manifest,
                            ],
                        )?;
                        let mut graph: Metadata =
                            decode_metadata(&output).map_err(|e| format!("{manifest}: {e}"))?;
                        for package in &mut graph.packages {
                            package.manifest_path = relative(root, &package.manifest_path);
                            for target in &mut package.targets {
                                target.src_path = relative(root, &target.src_path);
                            }
                        }
                        Ok((manifest.clone(), graph))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("metadata reader panicked"))
                .collect::<Result<BTreeMap<_, _>, String>>()
        })?;
        Ok(Self {
            files,
            paths,
            manifests,
            graphs,
            config,
        })
    }
    pub fn text(&self, name: &str) -> &str {
        self.files.get(name).map_or("", String::as_str)
    }
    pub fn courts(&self) -> Vec<(&str, &str)> {
        self.text(&format!("{CONFIG}test-workspaces.txt"))
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter_map(|l| l.split_once('|'))
            .map(|(a, b)| (a.trim(), b.trim()))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_decoding_requires_a_resolved_graph_and_named_packages() {
        let bytes = br#"{"packages":[{"id":"one","name":"package","manifest_path":"Cargo.toml","dependencies":[],"targets":[]}],"workspace_members":["one"],"resolve":{"nodes":[{"id":"one","deps":[]}]}}"#;
        let graph = decode_metadata(bytes).unwrap();
        assert_eq!(graph.packages[0].name, "package");
        assert_eq!(graph.workspace_members, vec!["one"]);
        assert!(decode_metadata(b"{}").is_err());
    }
}
fn relative(root: &Path, name: &str) -> String {
    let root = root
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('\\', "/");
    let name = name.replace('\\', "/");
    if name
        .to_lowercase()
        .starts_with(&format!("{}/", root.to_lowercase()))
    {
        name[root.len() + 1..].into()
    } else {
        name
    }
}
/// Decode only graph fields the rules consume; no procedural-macro dependency.
fn decode_metadata(bytes: &[u8]) -> Result<Metadata, String> {
    use crate::config::{Dependency, Edge, Kind, Node, Package, Resolve, Target};
    use serde_json::Value;
    fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
        v[k].as_array()
            .ok_or_else(|| format!("metadata array missing: {k}"))
    }
    fn text(v: &Value, k: &str) -> Result<String, String> {
        v[k].as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("metadata string missing: {k}"))
    }
    fn names(v: &Value, k: &str) -> Result<Vec<String>, String> {
        array(v, k)?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("metadata string array invalid: {k}"))
            })
            .collect()
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let packages = array(&v, "packages")?
        .iter()
        .map(|p| {
            let dependencies = array(p, "dependencies")?
                .iter()
                .map(|d| {
                    Ok(Dependency {
                        name: text(d, "name")?,
                        kind: d["kind"].as_str().map(str::to_owned),
                        req: text(d, "req")?,
                        uses_default_features: d["uses_default_features"]
                            .as_bool()
                            .ok_or("metadata default-feature flag missing")?,
                        features: names(d, "features")?,
                    })
                })
                .collect::<Result<_, String>>()?;
            let targets = array(p, "targets")?
                .iter()
                .map(|t| {
                    Ok(Target {
                        name: text(t, "name")?,
                        kind: names(t, "kind")?,
                        src_path: text(t, "src_path")?,
                    })
                })
                .collect::<Result<_, String>>()?;
            Ok(Package {
                id: text(p, "id")?,
                name: text(p, "name")?,
                manifest_path: text(p, "manifest_path")?,
                dependencies,
                targets,
            })
        })
        .collect::<Result<_, String>>()?;
    let nodes = array(&v["resolve"], "nodes")?
        .iter()
        .map(|n| {
            let deps = array(n, "deps")?
                .iter()
                .map(|d| {
                    let dep_kinds = array(d, "dep_kinds")?
                        .iter()
                        .map(|k| Kind {
                            kind: k["kind"].as_str().map(str::to_owned),
                        })
                        .collect();
                    Ok(Edge {
                        pkg: text(d, "pkg")?,
                        dep_kinds,
                    })
                })
                .collect::<Result<_, String>>()?;
            Ok(Node {
                id: text(n, "id")?,
                deps,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(Metadata {
        packages,
        workspace_members: names(&v, "workspace_members")?,
        resolve: Some(Resolve { nodes }),
    })
}
#[cfg(test)]
impl Inputs {
    pub fn fixture(files: &[(&str, &str)], config: &str) -> Self {
        Self {
            files: files
                .iter()
                .map(|(p, t)| (p.to_string(), t.to_string()))
                .collect(),
            paths: files.iter().map(|(p, _)| p.to_string()).collect(),
            manifests: files
                .iter()
                .filter(|(p, _)| p.ends_with("Cargo.toml"))
                .map(|(p, t)| (p.to_string(), toml::from_str(t).unwrap()))
                .collect(),
            graphs: BTreeMap::new(),
            config: toml::from_str(config).unwrap(),
        }
    }
}
