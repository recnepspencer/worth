//! Compiled fixture region existence and source containment.
use super::configuration::{confined_path, SOURCE};
use super::{native_test_cfg, region_compilation};
use crate::source_rules::{parse_crate_modules_where, GovernedCrate};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct FixtureSources {
    reachable: Option<Result<BTreeSet<PathBuf>, String>>,
}

impl FixtureSources {
    pub(super) fn validate(&mut self, root: &Path, name: &str) -> Result<(), String> {
        if !confined_path(name) || !name.ends_with(".rs") {
            return Err("compiled source must be a Rust path beneath topology_entry/src".into());
        }
        let owner = root
            .join(SOURCE)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let source = owner
            .join(name)
            .canonicalize()
            .map_err(|error| format!("compiled source missing: {error}"))?;
        if !source.starts_with(&owner) {
            return Err("compiled source escapes topology_entry/src".into());
        }
        let reachable = self.reachable.get_or_insert_with(|| native_sources(&owner));
        if !reachable.as_ref().map_err(Clone::clone)?.contains(&source) {
            return Err("compiled source is absent from the topology fixture module graph".into());
        }
        let text =
            crate::source_documents::read_to_string(&source).map_err(|error| error.to_string())?;
        let text = text.replace("\r\n", "\n");
        let begin = "// compiled-example: begin\n";
        let end = "// compiled-example: end\n";
        if text.matches(begin).count() != 1 || text.matches(end).count() != 1 {
            return Err("compiled source must contain exactly one begin/end region".into());
        }
        if text.find(begin) >= text.find(end) {
            return Err("compiled source region ends before it begins".into());
        }
        let syntax =
            crate::source_syntax::parse_named(&source, &text).map_err(|error| error.to_string())?;
        let first_line = text[..text.find(begin).unwrap()]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 2;
        let last_line = text[..text.find(end).unwrap()]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        region_compilation::validate(&syntax, first_line, last_line)?;
        Ok(())
    }
}

fn native_sources(owner: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let crate_root = owner.parent().unwrap().to_path_buf();
    let graph = parse_crate_modules_where(
        &GovernedCrate {
            package: "worth-query-topology-entry".into(),
            crate_root: crate_root.clone(),
            relative_crate_root: SOURCE.trim_end_matches("/src").into(),
        },
        &|module| native_test_cfg::active(&module.attrs) == Ok(true),
    )?;
    graph
        .modules
        .iter()
        .filter(|(path, _)| {
            (0..=path.len()).all(|length| {
                graph.modules.get(&path[..length]).is_some_and(|ancestor| {
                    native_test_cfg::active(&ancestor.attributes) == Ok(true)
                })
            })
        })
        .map(|(_, node)| {
            crate_root
                .join(&node.relative_source)
                .canonicalize()
                .map_err(|error| error.to_string())
        })
        .collect()
}
