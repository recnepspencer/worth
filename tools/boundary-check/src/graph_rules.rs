//! Metadata dependency policy and court target topology.
use crate::{config::*, inputs::Inputs, text_rules::manifest_features};
use std::collections::{BTreeMap, BTreeSet};
mod frontier;
mod splits;
use frontier::{frontiers, surface};

pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let owned: BTreeSet<_> = input
        .manifests
        .iter()
        .filter(|(p, _)| governed(p))
        .filter_map(|(_, v)| v.get("package"))
        .map(|p| string(p, "name").to_owned())
        .collect();
    for (path, manifest) in &input.manifests {
        let mut names = owned.clone();
        if let Some((_, workspace)) = input
            .manifests
            .iter()
            .filter(|(p, v)| {
                v.get("workspace").is_some()
                    && (p.as_str() == "Cargo.toml"
                        || under(path, p.trim_end_matches("/Cargo.toml")))
            })
            .max_by_key(|(p, _)| p.len())
        {
            if let Some(aliases) = workspace["workspace"]
                .get("dependencies")
                .and_then(toml::Value::as_table)
            {
                for (alias, dependency) in aliases {
                    if owned.contains(
                        dependency
                            .get("package")
                            .and_then(toml::Value::as_str)
                            .unwrap_or(alias),
                    ) {
                        names.insert(alias.clone());
                    }
                }
            }
        }
        out.extend(manifest_features(path, manifest, &names));
    }
    for key in [
        "dependency_denials",
        "source_dependency_allowlists",
        "dependency_target_allowlists",
    ] {
        for rule in table(&input.config, key) {
            let Some(graph) = input.graphs.get(string(rule, "workspace_manifest")) else {
                out.push(diagnostic(
                    "BC2001",
                    key,
                    "configured workspace metadata is missing",
                ));
                continue;
            };
            for package in &graph.packages {
                out.extend(dependencies(key, rule, package));
            }
        }
    }
    let splits =
        toml::from_str::<toml::Value>(input.text(&format!("{CONFIG}court-frontier-splits.toml")));
    let splits = match splits {
        Ok(v) => v,
        Err(e) => {
            out.push(diagnostic("BC9002", CONFIG, e.to_string()));
            return out;
        }
    };
    out.extend(splits::check(&splits));
    let mut registered = BTreeSet::new();
    for (workspace, stem) in input.courts() {
        let Some(graph) = input.graphs.get(&format!("{workspace}/Cargo.toml")) else {
            out.push(diagnostic(
                "BC9002",
                workspace,
                "workspace metadata missing",
            ));
            continue;
        };
        let frontiers = frontiers(graph);
        let mut groups: BTreeMap<Vec<String>, Vec<&Package>> = BTreeMap::new();
        for p in graph
            .packages
            .iter()
            .filter(|p| graph.workspace_members.contains(&p.id))
        {
            let Some(manifest) = input.manifests.get(&p.manifest_path) else {
                continue;
            };
            let directory = p.manifest_path.trim_end_matches("/Cargo.toml");
            let worth = manifest
                .get("package")
                .and_then(|p| p.get("metadata"))
                .and_then(|m| m.get("worth"));
            let court =
                p.name.contains("-court-") || worth.is_some_and(|m| m.get("court").is_some());
            let testkit =
                p.name.contains("-testkit") || worth.is_some_and(|m| m.get("testkit").is_some());
            if p.dependencies.iter().any(|d| d.name.contains("-court-")) {
                out.push(diagnostic(
                    "BC9002",
                    &p.manifest_path,
                    "court must be a leaf, including dev dependencies",
                ));
            }
            if court {
                let tests: Vec<_> = p
                    .targets
                    .iter()
                    .filter(|t| t.kind.contains(&"test".into()))
                    .collect();
                let canonical = p.name.replace('-', "_");
                let test = table(manifest, "test");
                if manifest["package"]
                    .get("autotests")
                    .and_then(toml::Value::as_bool)
                    != Some(false)
                    || tests.len() != 1
                    || test.len() != 1
                    || tests.iter().any(|t| {
                        t.name != canonical
                            || t.src_path != format!("{directory}/tests/{canonical}.rs")
                            || !input.paths.contains(&t.src_path)
                    })
                    || test.iter().any(|t| {
                        t.get("harness").and_then(toml::Value::as_bool) == Some(false)
                            || t.get("test").and_then(toml::Value::as_bool) == Some(false)
                    })
                {
                    out.push(diagnostic("BC9002", &p.manifest_path, "court requires one explicit canonical standard-harness test target and autotests=false"));
                }
                if p.targets.iter().any(|t| t.kind.iter().any(|k| k != "test"))
                    || input.paths.iter().any(|f| {
                        under(f, directory)
                            && f.ends_with(".rs")
                            && !under(f, &format!("{directory}/tests"))
                    })
                {
                    out.push(diagnostic(
                        "BC9002",
                        &p.manifest_path,
                        "court cannot contain production targets or Rust outside tests/",
                    ));
                }
                let frontier = frontiers.get(&p.id).cloned().unwrap_or_default();
                let base = format!("{stem}-court-{}", surface(stem, &frontier));
                let split = table(&splits, "splits")
                    .into_iter()
                    .find(|s| string(s, "court") == p.name);
                let expected = split.map_or(base.clone(), |s| {
                    if base == p.name {
                        base.clone()
                    } else {
                        format!("{base}-{}", string(s, "area"))
                    }
                });
                if let Some(split) = split {
                    registered.insert(p.name.clone());
                    if strings(split, "frontier") != frontier
                        || strings(split, "owners").is_empty()
                        || string(split, "area").is_empty()
                    {
                        out.push(diagnostic(
                            "BC9002",
                            &p.manifest_path,
                            "invalid frontier split registration",
                        ));
                    }
                }
                if p.name != expected {
                    out.push(diagnostic(
                        "BC9002",
                        &p.manifest_path,
                        format!("frontier naming requires `{expected}`"),
                    ));
                }
                groups.entry(frontier).or_default().push(p);
                for path in input
                    .paths
                    .iter()
                    .filter_map(|f| f.strip_prefix(&format!("{directory}/tests/{canonical}/")))
                {
                    if !category_valid(path) {
                        out.push(diagnostic(
                            "BC9002",
                            &format!("{directory}/tests/{canonical}/{path}"),
                            "category must be a top-level path segment",
                        ));
                    }
                }
            } else if input
                .paths
                .iter()
                .any(|p| under(p, &format!("{directory}/tests")))
                || manifest.get("test").is_some()
            {
                out.push(diagnostic(
                    "BC9002",
                    &p.manifest_path,
                    "product/testkit cannot contain tests/ or integration targets",
                ));
            }
            if testkit
                && !(p.name == format!("{stem}-testkit")
                    || p.name.starts_with(&format!("{stem}-testkit-")))
            {
                out.push(diagnostic("BC9002", &p.manifest_path, "misnamed testkit"));
            }
        }
        for courts in groups.values() {
            if courts.len() > 1 && courts.iter().any(|p| !registered.contains(&p.name)) {
                out.push(diagnostic(
                    "BC9002",
                    workspace,
                    "equal product frontiers require registered area splits",
                ));
            }
        }
    }
    for split in table(&splits, "splits") {
        if !registered.contains(string(split, "court")) {
            out.push(diagnostic(
                "BC9002",
                CONFIG,
                "frontier split names an absent court",
            ));
        }
    }
    out
}
fn selected(rule: &toml::Value, package: &str, names: &str, prefixes: &str) -> bool {
    rule.get(names)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .any(|s| s.as_str() == Some(package))
        || rule
            .get(prefixes)
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .any(|p| package.starts_with(p))
}
fn dependencies(key: &str, rule: &toml::Value, p: &Package) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let governs_source = match key {
        "dependency_denials" => selected(rule, &p.name, "sources", "source_prefixes"),
        "source_dependency_allowlists" => strings(rule, "sources").contains(&p.name),
        "dependency_target_allowlists" => strings(rule, "governed_source_prefixes")
            .iter()
            .any(|prefix| p.name.starts_with(prefix)),
        _ => unreachable!(),
    };
    if !governs_source {
        return out;
    }
    for d in &p.dependencies {
        let denied = match key {
            "dependency_denials" => selected(
                rule,
                &d.name,
                "forbidden_targets",
                "forbidden_target_prefixes",
            ),
            "source_dependency_allowlists" => !strings(rule, "allowed_targets").contains(&d.name),
            "dependency_target_allowlists" => {
                d.name == string(rule, "target")
                    && !strings(rule, "allowed_sources").contains(&p.name)
            }
            _ => unreachable!(),
        };
        if denied {
            out.push(diagnostic(
                "BC2001",
                &p.manifest_path,
                format!("{} -> {}: {}", p.name, d.name, string(rule, "guidance")),
            ));
        }
    }
    if key == "source_dependency_allowlists" {
        for contract in table(rule, "dependency_contracts") {
            if !p.dependencies.iter().any(|d| {
                d.name == string(contract, "target")
                    && d.req == string(contract, "version_requirement")
                    && Some(d.uses_default_features)
                        == contract
                            .get("uses_default_features")
                            .and_then(toml::Value::as_bool)
                    && d.features.iter().collect::<BTreeSet<_>>()
                        == strings(contract, "features").iter().collect()
            }) {
                out.push(diagnostic(
                    "BC2001",
                    &p.manifest_path,
                    "dependency contract mismatch",
                ));
            }
        }
    }
    out
}
fn category_valid(path: &str) -> bool {
    let segments: Vec<_> = path
        .trim_end_matches(".rs")
        .split('/')
        .filter(|s| *s != "mod")
        .collect();
    !segments.iter().enumerate().any(|(i, s)| {
        ["fast", "exhaustive", "scale", "gpu"].contains(s)
            || i > 0 && ["process", "oracle", "cost", "determinism", "compile_fail"].contains(s)
            || *s == "serial" && (i != 1 || !["process", "compile_fail"].contains(&segments[0]))
    })
}
#[cfg(test)]
mod tests;
