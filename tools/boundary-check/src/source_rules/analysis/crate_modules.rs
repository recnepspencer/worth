//! Parse each governed crate's module graph once (filesystem + syn only).
//!
//! The graph tracks the library target Cargo compiles and every rustc-selectable
//! module body: `[lib] path`, virtual-directory `#[path]` (including nested
//! inline parents), `cfg_attr(..., path = ...)`, and cfg-exclusive duplicates
//! merged fail-closed.

use super::module_source::{
    all_path_selectors, child_module_dir, directory_after_loading_file, path_attribute_dir,
    resolve_child_sources,
};
use crate::cargo_graph::normalize_path;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Attribute, File, Item, ItemMod, Visibility};

/// One governed crate discovered under configured subworkspace roots.
#[derive(Clone, Debug)]
pub(crate) struct GovernedCrate {
    pub(crate) package: String,
    pub(crate) crate_root: PathBuf,
    pub(crate) relative_crate_root: String,
}

/// Parsed module graph keyed by module path segments from the crate root.
pub(crate) struct ModuleGraph {
    pub(crate) modules: BTreeMap<Vec<String>, ModuleNode>,
}

/// One module's items, visibility from its parent, and source path for diagnostics.
pub(crate) struct ModuleNode {
    pub(crate) relative_source: String,
    pub(crate) public_from_parent: bool,
    /// A file module's inner attributes. An inline module's are on its
    /// declaration, which its parent's items hold.
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) items: Vec<Item>,
}

/// Which child module declarations a graph walk descends into.
pub(crate) type ModuleFilter<'a> = &'a dyn Fn(&ItemMod) -> bool;

pub(super) fn parse_crate_modules(governed: &GovernedCrate) -> Result<ModuleGraph, String> {
    parse_crate_modules_where(governed, &|_| true)
}

/// The library graph without the child modules `descend` rejects (their
/// declarations stay in the parent's items).
pub(crate) fn parse_crate_modules_where(
    governed: &GovernedCrate,
    descend: ModuleFilter<'_>,
) -> Result<ModuleGraph, String> {
    let lib_source = super::library_target::resolve_lib_source_path(&governed.crate_root)?;
    if !lib_source.is_file() {
        return Err(format!(
            "governed crate {} library target missing: {}",
            governed.package,
            lib_source.display()
        ));
    }
    let root_dir = lib_source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut modules = BTreeMap::new();
    parse_module_tree(
        &governed.crate_root,
        &lib_source,
        &root_dir,
        Vec::new(),
        true,
        &mut modules,
        descend,
    )?;
    Ok(ModuleGraph { modules })
}

/// Parse non-library Cargo source targets for import-shape enforcement.
///
/// Public API reachability remains rooted in the library graph. These nodes
/// cover source text compiled through dev/build/bin/example/bench targets.
pub(super) fn parse_additional_source_targets(
    governed: &GovernedCrate,
) -> Result<Vec<ModuleNode>, String> {
    let mut paths = Vec::new();
    for relative in ["build.rs", "src/main.rs"] {
        let path = governed.crate_root.join(relative);
        if path.is_file() {
            paths.push(path);
        }
    }
    for relative in ["src/bin", "tests", "examples", "benches"] {
        collect_rust_files(&governed.crate_root.join(relative), &mut paths)?;
    }
    collect_manifest_target_paths(governed, &mut paths)?;
    paths.sort();
    paths.dedup();

    let mut nodes = Vec::new();
    for path in paths {
        let root_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let mut target_modules = BTreeMap::new();
        parse_module_tree(
            &governed.crate_root,
            &path,
            &root_dir,
            Vec::new(),
            false,
            &mut target_modules,
            &|_| true,
        )?;
        nodes.extend(target_modules.into_values());
    }
    Ok(nodes)
}

pub(super) fn parse_additional_production_targets(
    governed: &GovernedCrate,
) -> Result<Vec<ModuleNode>, String> {
    Ok(parse_additional_production_target_graphs(governed)?
        .into_iter()
        .flat_map(|graph| graph.modules.into_values())
        .collect())
}

/// One module graph per non-library production target, rooted at its source.
pub(super) fn parse_additional_production_target_graphs(
    governed: &GovernedCrate,
) -> Result<Vec<ModuleGraph>, String> {
    let source_root = governed.crate_root.join("src");
    let mut paths = Vec::new();
    let main = source_root.join("main.rs");
    if main.is_file() {
        paths.push(main);
    }
    collect_rust_files(&source_root.join("bin"), &mut paths)?;
    collect_manifest_target_paths(governed, &mut paths)?;
    paths.retain(|path| path.starts_with(&source_root));
    paths.sort();
    paths.dedup();

    let mut graphs = Vec::new();
    for path in paths {
        let root_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let mut modules = BTreeMap::new();
        parse_module_tree(
            &governed.crate_root,
            &path,
            &root_dir,
            Vec::new(),
            false,
            &mut modules,
            &|_| true,
        )?;
        graphs.push(ModuleGraph { modules });
    }
    Ok(graphs)
}

fn collect_manifest_target_paths(
    governed: &GovernedCrate,
    paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let manifest_path = governed.crate_root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let manifest: toml::Value = toml::from_str(&text)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    if let Some(build) = manifest
        .get("package")
        .and_then(|package| package.get("build"))
        .and_then(|build| build.as_str())
    {
        paths.push(governed.crate_root.join(build));
    }
    for table_name in ["bin", "example", "test", "bench"] {
        let Some(targets) = manifest.get(table_name).and_then(|value| value.as_array()) else {
            continue;
        };
        for target in targets {
            if let Some(path) = target
                .as_table()
                .and_then(|table| table.get("path"))
                .and_then(|path| path.as_str())
            {
                paths.push(governed.crate_root.join(path));
            }
        }
    }
    Ok(())
}

fn collect_rust_files(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| {
        format!(
            "read source target directory {}: {error}",
            directory.display()
        )
    })? {
        let path = entry
            .map_err(|error| format!("read source target entry: {error}"))?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
    Ok(())
}

fn parse_module_tree(
    crate_root: &Path,
    source_path: &Path,
    module_dir: &Path,
    module_path: Vec<String>,
    public_from_parent: bool,
    modules: &mut BTreeMap<Vec<String>, ModuleNode>,
    descend: ModuleFilter<'_>,
) -> Result<(), String> {
    let text = fs::read_to_string(source_path)
        .map_err(|e| format!("read {}: {e}", source_path.display()))?;
    let file: File = crate::source_syntax::parse_file(&text)
        .map_err(|e| format!("parse {}: {e}", source_path.display()))?;
    let relative_source = relative_under_crate(crate_root, source_path)?;

    let mut child_mods = Vec::new();
    for item in &file.items {
        if let Item::Mod(item_mod) = item {
            if descend(item_mod) {
                child_mods.push(item_mod.clone());
            }
        }
    }

    merge_module_node(
        modules,
        module_path.clone(),
        relative_source,
        public_from_parent,
        file.attrs,
        file.items,
    );

    for item_mod in child_mods {
        load_child_module(
            crate_root,
            source_path,
            module_dir,
            &module_path,
            &item_mod,
            modules,
            descend,
        )?;
    }
    Ok(())
}

fn load_child_module(
    crate_root: &Path,
    parent_source: &Path,
    parent_dir: &Path,
    parent_path: &[String],
    item_mod: &ItemMod,
    modules: &mut BTreeMap<Vec<String>, ModuleNode>,
    descend: ModuleFilter<'_>,
) -> Result<(), String> {
    let name = item_mod.ident.to_string();
    let mut child_path = parent_path.to_vec();
    child_path.push(name.clone());
    let public_from_parent = is_public_visibility(&item_mod.vis);
    // A raw keyword module (`mod r#async;`) lives in `async.rs`.
    let file_name = name.strip_prefix("r#").unwrap_or(&name);
    let child_dir = child_module_dir(parent_dir, file_name);

    if let Some((_, items)) = &item_mod.content {
        let relative_source = relative_under_crate(crate_root, parent_source)?;
        let mut nested = Vec::new();
        for item in items {
            if let Item::Mod(nested_mod) = item {
                if descend(nested_mod) {
                    nested.push(nested_mod.clone());
                }
            }
        }
        merge_module_node(
            modules,
            child_path.clone(),
            relative_source,
            public_from_parent,
            Vec::new(),
            items.clone(),
        );
        // Nested declarations resolve against this inline module's virtual directory.
        for nested_mod in nested {
            load_child_module(
                crate_root,
                parent_source,
                &child_dir,
                &child_path,
                &nested_mod,
                modules,
                descend,
            )?;
        }
        return Ok(());
    }

    let path_parent_dir = path_attribute_dir(parent_source, parent_dir);
    let path_selected_sources = all_path_selectors(&item_mod.attrs)
        .into_iter()
        .map(|selector| path_parent_dir.join(selector))
        .collect::<Vec<_>>();
    let sources = resolve_child_sources(parent_dir, &path_parent_dir, file_name, &item_mod.attrs)?;
    for source in sources {
        let loaded_dir = if path_selected_sources.contains(&source) {
            source
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            directory_after_loading_file(&source)
        };
        parse_module_tree(
            crate_root,
            &source,
            &loaded_dir,
            child_path.clone(),
            public_from_parent,
            modules,
            descend,
        )?;
    }
    Ok(())
}

fn merge_module_node(
    modules: &mut BTreeMap<Vec<String>, ModuleNode>,
    module_path: Vec<String>,
    relative_source: String,
    public_from_parent: bool,
    attributes: Vec<Attribute>,
    items: Vec<Item>,
) {
    if let Some(existing) = modules.get_mut(&module_path) {
        existing.attributes.extend(attributes);
        existing.items.extend(items);
        if !existing.relative_source.contains(&relative_source) {
            existing.relative_source = format!("{};{}", existing.relative_source, relative_source);
        }
        existing.public_from_parent = existing.public_from_parent || public_from_parent;
    } else {
        modules.insert(
            module_path,
            ModuleNode {
                relative_source,
                public_from_parent,
                attributes,
                items,
            },
        );
    }
}

fn relative_under_crate(crate_root: &Path, source_path: &Path) -> Result<String, String> {
    let relative = source_path
        .strip_prefix(crate_root)
        .map_err(|e| format!("strip crate root from {}: {e}", source_path.display()))?;
    Ok(normalize_path(relative))
}

pub(super) fn is_public_visibility(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

pub(super) fn module_path_display(path: &[String]) -> String {
    if path.is_empty() {
        return "crate".to_owned();
    }
    path.join("::")
}
