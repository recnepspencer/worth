//! Resolve every name of every facade snapshot row to its definitions and
//! record whether each definition carries rustdoc.

use super::doc_presence::{missing_doc, MissingDoc};
use super::resolution::{CrateIndex, Definition, Resolver};
use crate::manifest_types::Road1Package;
use crate::snapshots::document::FacadeDocument;
use std::collections::BTreeMap;
use std::path::Path;

/// Documentation state of one exported facade name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NameStatus {
    Documented,
    /// Defined by a registry, git, or standard-library crate: out of scope.
    OutsideWorkspace,
    /// At least one workspace definition lacks rustdoc.
    Undocumented(Vec<MissingDoc>),
    /// The resolver could not attribute the name to a definition.
    Unresolved(String),
}

/// Facade row label to exported name to documentation state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FacadeDocObservation {
    pub(crate) facades: BTreeMap<String, BTreeMap<String, NameStatus>>,
}

impl FacadeDocObservation {
    pub(crate) fn status(&self, package: &str, name: &str) -> Option<&NameStatus> {
        self.facades.get(package).and_then(|names| names.get(name))
    }
}

/// Observe rustdoc presence for each name in `facades`.
///
/// A row label is a governed package name, optionally followed by `::` and
/// a namespace path under that package's `facade` module.
pub(crate) fn observe_facade_docs(
    root: &Path,
    packages: &[Road1Package],
    facades: &FacadeDocument,
) -> Result<FacadeDocObservation, String> {
    let crates = CrateIndex::new(root);
    let resolver = Resolver::new(&crates);
    let mut observation = FacadeDocObservation::default();
    for row in &facades.facades {
        let (krate, module) = facade_module(&resolver, packages, &row.package)?;
        let names = row
            .exports
            .iter()
            .map(|name| {
                let segments = name.split("::").map(str::to_owned).collect::<Vec<_>>();
                let status = match resolver.resolve_path(krate, &module, &segments) {
                    Ok(definitions) => status(&resolver, &definitions),
                    Err(error) => NameStatus::Unresolved(error),
                };
                (name.clone(), status)
            })
            .collect();
        observation.facades.insert(row.package.clone(), names);
    }
    Ok(observation)
}

fn facade_module(
    resolver: &Resolver<'_>,
    packages: &[Road1Package],
    label: &str,
) -> Result<(usize, Vec<String>), String> {
    let mut segments = label.split("::");
    let package_name = segments.next().unwrap_or(label);
    let package = packages
        .iter()
        .find(|package| package.name == package_name)
        .ok_or_else(|| format!("facade row {label} names no governed package"))?;
    let crate_root = Path::new(&package.manifest_path)
        .parent()
        .ok_or_else(|| format!("{} has no crate directory", package.manifest_path))?;
    let krate = resolver.crates().load(crate_root)?;
    let mut path = vec!["facade".to_owned()];
    path.extend(segments.map(str::to_owned));
    match resolver.resolve_path(krate, &[], &path)?.as_slice() {
        [Definition::Module { krate, path }] => Ok((*krate, path.clone())),
        other => Err(format!(
            "facade row {label} must resolve to exactly one module, found {other:?}"
        )),
    }
}

fn status(resolver: &Resolver<'_>, definitions: &[Definition]) -> NameStatus {
    if definitions.is_empty() {
        return NameStatus::Unresolved("no definition found".to_owned());
    }
    if definitions
        .iter()
        .all(|definition| matches!(definition, Definition::OutsideWorkspace(_)))
    {
        return NameStatus::OutsideWorkspace;
    }
    let mut missing = definitions
        .iter()
        .filter_map(|definition| missing_doc(resolver, definition))
        .collect::<Vec<_>>();
    missing.sort();
    missing.dedup();
    if missing.is_empty() {
        NameStatus::Documented
    } else {
        NameStatus::Undocumented(missing)
    }
}
