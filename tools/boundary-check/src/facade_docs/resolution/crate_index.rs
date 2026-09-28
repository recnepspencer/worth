//! Lazily parsed library crates a facade name may be defined in.
//!
//! A crate enters the index when a facade row names its package or when a
//! `use` path reaches it through a path-backed dependency of an indexed crate.
//! Each crate is parsed once through the shared module-graph loader, without
//! its `#[cfg(test)]` modules.

use super::item_shape::{is_macro_export, is_test_only};
use crate::cargo_graph::{normalize_path, package_name_from_manifest};
use crate::source_rules::{
    parse_crate_modules_where, path_backed_dependency_roots, GovernedCrate, ModuleGraph,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use syn::Item;

/// Position of a crate inside one [`CrateIndex`].
pub(crate) type CrateId = usize;

/// One parsed library crate and the names it can reach at a path start.
pub(crate) struct IndexedCrate {
    pub(crate) repo_relative_root: String,
    pub(crate) graph: ModuleGraph,
    /// `#[macro_export]` definitions: defining module path and item index.
    pub(crate) exported_macros: Vec<(Vec<String>, usize)>,
    path_dependencies: BTreeMap<String, PathBuf>,
    external_dependencies: BTreeSet<String>,
}

/// How a path-start crate name resolves from inside one crate.
pub(crate) enum ExternCrate {
    Workspace(CrateId),
    /// A registry, git, or standard-library crate: never a workspace definition.
    OutsideWorkspace,
    Unknown,
}

pub(crate) struct CrateIndex {
    repository_root: PathBuf,
    crates: RefCell<Vec<Rc<IndexedCrate>>>,
    by_root: RefCell<BTreeMap<PathBuf, CrateId>>,
}

impl CrateIndex {
    pub(crate) fn new(repository_root: &Path) -> Self {
        Self {
            repository_root: canonical(repository_root),
            crates: RefCell::new(Vec::new()),
            by_root: RefCell::new(BTreeMap::new()),
        }
    }

    pub(crate) fn crate_at(&self, id: CrateId) -> Rc<IndexedCrate> {
        Rc::clone(&self.crates.borrow()[id])
    }

    /// Index the library crate whose manifest directory is `crate_root`.
    pub(crate) fn load(&self, crate_root: &Path) -> Result<CrateId, String> {
        let crate_root = canonical(crate_root);
        if let Some(id) = self.by_root.borrow().get(&crate_root) {
            return Ok(*id);
        }
        let indexed = parse_indexed_crate(&self.repository_root, &crate_root)?;
        let mut crates = self.crates.borrow_mut();
        crates.push(Rc::new(indexed));
        let id = crates.len() - 1;
        self.by_root.borrow_mut().insert(crate_root, id);
        Ok(id)
    }

    /// Resolve a path-start crate name as seen from inside crate `from`.
    pub(crate) fn extern_crate(&self, from: CrateId, name: &str) -> Result<ExternCrate, String> {
        let krate = self.crate_at(from);
        if let Some(dependency_root) = krate.path_dependencies.get(name) {
            return self.load(dependency_root).map(ExternCrate::Workspace);
        }
        if is_standard_crate(name) || krate.external_dependencies.contains(name) {
            return Ok(ExternCrate::OutsideWorkspace);
        }
        Ok(ExternCrate::Unknown)
    }

    /// Path-backed dependencies of `from`, loaded, for `$crate`-less macro search.
    pub(crate) fn loaded_dependencies(&self, from: CrateId) -> Result<Vec<CrateId>, String> {
        let roots = self
            .crate_at(from)
            .path_dependencies
            .values()
            .cloned()
            .collect::<Vec<_>>();
        roots.iter().map(|root| self.load(root)).collect()
    }
}

fn parse_indexed_crate(repository_root: &Path, crate_root: &Path) -> Result<IndexedCrate, String> {
    let manifest = crate_root.join("Cargo.toml");
    let repo_relative_root = crate_root
        .strip_prefix(repository_root)
        .map(normalize_path)
        .unwrap_or_else(|_| normalize_path(crate_root));
    let governed = GovernedCrate {
        package: package_name_from_manifest(&manifest)?,
        crate_root: crate_root.to_path_buf(),
        relative_crate_root: repo_relative_root.clone(),
    };
    // Test-only modules never define a facade item; skipping them halves parsing.
    let graph = parse_crate_modules_where(&governed, &|module| !is_test_only(&module.attrs))?;
    let exported_macros = exported_macro_definitions(&graph);
    Ok(IndexedCrate {
        repo_relative_root,
        graph,
        exported_macros,
        path_dependencies: path_backed_dependency_roots(crate_root)?,
        external_dependencies: manifest_dependency_idents(&manifest)?,
    })
}

fn exported_macro_definitions(graph: &ModuleGraph) -> Vec<(Vec<String>, usize)> {
    graph
        .modules
        .iter()
        .flat_map(|(path, node)| {
            node.items
                .iter()
                .enumerate()
                .filter(|(_, item)| is_exported_macro_definition(item))
                .map(|(index, _)| (path.clone(), index))
        })
        .collect()
}

fn is_exported_macro_definition(item: &Item) -> bool {
    matches!(item, Item::Macro(definition)
        if definition.ident.is_some() && is_macro_export(&definition.attrs))
}

/// Every dependency key a crate's manifest declares, as Rust idents.
fn manifest_dependency_idents(manifest: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(manifest)
        .map_err(|error| format!("read {}: {error}", manifest.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", manifest.display()))?;
    let mut tables = Vec::new();
    tables.extend(value.get("dependencies").and_then(|table| table.as_table()));
    if let Some(targets) = value.get("target").and_then(|table| table.as_table()) {
        tables.extend(
            targets
                .values()
                .filter_map(|target| target.get("dependencies").and_then(|t| t.as_table())),
        );
    }
    Ok(tables
        .into_iter()
        .flat_map(|table| table.keys())
        .map(|key| key.replace('-', "_"))
        .collect())
}

fn is_standard_crate(name: &str) -> bool {
    matches!(name, "std" | "core" | "alloc" | "proc_macro")
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
