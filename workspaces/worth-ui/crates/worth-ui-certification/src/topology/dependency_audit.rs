use std::collections::{HashMap, HashSet};
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{File, ItemExternCrate, ItemUse, UseTree};

use super::workspace_source_inventory::WorkspaceSourceInventory;

fn parse_rust_file(inventory: &WorkspaceSourceInventory, path: &Path) -> File {
    let text = inventory.text(path);
    syn::parse_file(text).unwrap_or_else(|error| {
        panic!("{} should parse as Rust source: {error}", path.display());
    })
}

#[derive(Default)]
struct AliasCollector {
    use_aliases: HashMap<String, Vec<String>>,
}

impl Visit<'_> for AliasCollector {
    fn visit_item_extern_crate(&mut self, item_extern_crate: &ItemExternCrate) {
        let alias = item_extern_crate
            .rename
            .as_ref()
            .map(|(_, ident)| ident)
            .unwrap_or(&item_extern_crate.ident)
            .to_string();
        self.use_aliases
            .insert(alias, vec![item_extern_crate.ident.to_string()]);
        visit::visit_item_extern_crate(self, item_extern_crate);
    }

    fn visit_item_use(&mut self, item_use: &ItemUse) {
        collect_use_aliases(&item_use.tree, &mut Vec::new(), &mut self.use_aliases);
        visit::visit_item_use(self, item_use);
    }
}

fn collect_use_aliases(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    aliases: &mut HashMap<String, Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_aliases(&path.tree, prefix, aliases);
            prefix.pop();
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_aliases(item, prefix, aliases);
            }
        }
        UseTree::Name(name) => {
            if !prefix.is_empty() {
                let mut full_path = prefix.clone();
                full_path.push(name.ident.to_string());
                aliases.insert(name.ident.to_string(), full_path);
            }
        }
        UseTree::Rename(rename) => {
            let mut full_path = prefix.clone();
            full_path.push(rename.ident.to_string());
            aliases.insert(rename.rename.to_string(), full_path);
        }
        _ => {}
    }
}

struct PathCollector<'a> {
    use_aliases: &'a HashMap<String, Vec<String>>,
    collected_paths: Vec<Vec<String>>,
}

impl<'a> Visit<'_> for PathCollector<'a> {
    fn visit_path(&mut self, path: &syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.collected_paths
            .push(expand_use_alias_path(segments, self.use_aliases));
        visit::visit_path(self, path);
    }
}

fn expand_use_alias_path(
    mut segments: Vec<String>,
    use_aliases: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut expanded_aliases = HashSet::new();

    loop {
        let Some(first) = segments.first().cloned() else {
            return segments;
        };
        let Some(alias_path) = use_aliases.get(&first) else {
            return segments;
        };
        if !expanded_aliases.insert(first) {
            return segments;
        }

        let mut expanded = alias_path.clone();
        expanded.extend(segments.into_iter().skip(1));
        segments = expanded;
    }
}

pub(crate) fn collect_file_paths(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<Vec<String>> {
    let parsed = parse_rust_file(inventory, path);
    let mut alias_collector = AliasCollector::default();
    alias_collector.visit_file(&parsed);

    let mut path_collector = PathCollector {
        use_aliases: &alias_collector.use_aliases,
        collected_paths: Vec::new(),
    };
    path_collector.visit_file(&parsed);
    path_collector.collected_paths
}

#[derive(Default)]
struct UsePathCollector {
    collected_paths: Vec<Vec<String>>,
}

impl Visit<'_> for UsePathCollector {
    fn visit_item_use(&mut self, item_use: &ItemUse) {
        collect_use_paths(&item_use.tree, Vec::new(), &mut self.collected_paths);
        visit::visit_item_use(self, item_use);
    }
}

pub(crate) fn collect_file_use_paths(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<Vec<String>> {
    let parsed = parse_rust_file(inventory, path);
    let mut collector = UsePathCollector::default();
    collector.visit_file(&parsed);
    collector.collected_paths
}

fn collect_use_paths(tree: &UseTree, prefix: Vec<String>, output: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            let mut next = prefix;
            next.push(path.ident.to_string());
            collect_use_paths(&path.tree, next, output);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_paths(item, prefix.clone(), output);
            }
        }
        UseTree::Name(name) => {
            let mut next = prefix;
            next.push(name.ident.to_string());
            output.push(next);
        }
        UseTree::Rename(rename) => {
            let mut next = prefix;
            next.push(rename.ident.to_string());
            output.push(next);
        }
        UseTree::Glob(_) => output.push(prefix),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManifestDependency {
    pub(crate) key: String,
    pub(crate) package: String,
}

pub(crate) fn manifests_dependencies(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<ManifestDependency> {
    let text = inventory.text(path);
    let manifest = text
        .parse::<toml::Table>().map(toml::Value::Table)
        .unwrap_or_else(|error| panic!("{} should parse as TOML: {error}", path.display()));

    ["dependencies", "dev-dependencies", "build-dependencies"]
        .into_iter()
        .filter_map(|section| manifest.get(section))
        .filter_map(toml::Value::as_table)
        .flat_map(|table| {
            table.iter().map(|(key, value)| ManifestDependency {
                key: key.clone(),
                package: value
                    .as_table()
                    .and_then(|entry| entry.get("package"))
                    .and_then(toml::Value::as_str)
                    .unwrap_or(key)
                    .to_string(),
            })
        })
        .collect()
}

pub(crate) fn manifest_dependency_crate_aliases(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> HashMap<String, String> {
    manifests_dependencies(inventory, path)
        .into_iter()
        .map(|dependency| {
            (
                dependency.key.replace('-', "_"),
                dependency.package.replace('-', "_"),
            )
        })
        .collect()
}

fn path_matches(segments: &[String], crate_name: &str, internal_root: &str) -> bool {
    segments.len() >= 2 && segments[0] == crate_name && segments[1] == internal_root
}

pub(crate) fn path_starts_with(segments: &[String], crate_name: &str) -> bool {
    segments
        .first()
        .is_some_and(|segment| segment == crate_name)
}

pub fn audit_no_cross_crate_deep_imports(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let crate_paths = [
        "crates/worth-ui/src",
        "crates/worth-ui-dsl/src",
        "crates/worth-ui-runtime/src",
        "crates/worth-ui-inspection/src",
        "crates/worth-ui-query-binding/src",
        "crates/worth-ui-host-contract/src",
        "crates/worth-ui-host-native/src",
        "crates/worth-ui-host-headless/src",
        "crates/worth-ui-certification/src",
    ];
    let forbidden_boundaries = [
        ("worth_ui_runtime", "lifecycle"),
        ("worth_ui_runtime", "source"),
        ("worth_ui_runtime", "host"),
        ("worth_ui_inspection", "facade"),
        ("worth_ui_inspection", "query"),
        ("worth_ui_inspection", "target"),
        ("worth_ui_inspection", "scope"),
        ("worth_ui_inspection", "receipt"),
        ("worth_ui_inspection", "posture"),
        ("worth_ui_dsl", "package"),
        ("worth_ui_dsl", "support"),
        ("worth_ui_host_contract", "runtime"),
        ("worth_ui_host_contract", "inspection"),
        ("worth_ui_query_binding", "facade"),
    ];
    let mut violations = Vec::new();
    let files = crate_paths
        .into_iter()
        .flat_map(|crate_path| inventory.rust_files_under(crate_path))
        .collect::<Vec<_>>();

    for file in files {
        for segments in collect_file_paths(inventory, file.absolute_path())
            .into_iter()
            .chain(collect_file_use_paths(inventory, file.absolute_path()))
        {
            for (crate_name, internal_root) in forbidden_boundaries {
                if path_matches(&segments, crate_name, internal_root) {
                    violations.push(format!(
                        "{} deep-imports `{crate_name}::{internal_root}` through structured Rust paths",
                        file.absolute_path().display()
                    ));
                }
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_non_product_crates_route_declaration_through_worth_ui_facade(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let crate_paths = [
        "crates/worth-ui-inspection/src",
        "crates/worth-ui-host-contract/src",
        "crates/worth-ui-host-native/src",
        "crates/worth-ui-host-headless/src",
        "crates/worth-ui-query-binding/src",
    ];
    let mut violations = Vec::new();
    let files = crate_paths
        .into_iter()
        .flat_map(|crate_path| inventory.rust_files_under(crate_path))
        .collect::<Vec<_>>();

    for file in files {
        for segments in collect_file_paths(inventory, file.absolute_path())
            .into_iter()
            .chain(collect_file_use_paths(inventory, file.absolute_path()))
        {
            if path_matches(&segments, "worth_ui_runtime", "facade")
                && segments
                    .get(2)
                    .is_some_and(|segment| segment == "declaration")
            {
                violations.push(format!(
                    "{} bypasses the product declaration facade and reaches `worth_ui_runtime::facade::declaration`; declaration consumers must enter through `worth_ui::facade::declaration`",
                    file.absolute_path().display()
                ));
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_host_adapter_dependency_boundary(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let mut violations = Vec::new();
    for root in [
        "crates/worth-ui-host-native",
        "crates/worth-ui-host-headless",
    ] {
        let cargo_toml = inventory.absolute_path(format!("{root}/Cargo.toml"));
        if !cargo_toml.exists() {
            continue;
        }
        let dependencies = manifests_dependencies(inventory, &cargo_toml);
        for forbidden_dep in ["worth-ui", "worth-ui-runtime", "worth-ui-inspection"] {
            if dependencies
                .iter()
                .any(|dependency| dependency.package == forbidden_dep)
            {
                violations.push(format!(
                    "{root} manifest must not depend on `{forbidden_dep}`"
                ));
            }
        }
        let aliases = manifest_dependency_crate_aliases(inventory, &cargo_toml);
        for file in inventory.rust_files_under(&format!("{root}/src")) {
            for segments in collect_file_paths(inventory, file.absolute_path()) {
                let normalized = normalize_manifest_alias_path(&segments, &aliases);
                if ["lifecycle", "source", "host"]
                    .into_iter()
                    .any(|module| path_matches(&normalized, "worth_ui_runtime", module))
                {
                    violations.push(format!(
                        "{} reaches worth-ui-runtime internals through structured Rust paths",
                        file.absolute_path().display()
                    ));
                }
                if path_starts_with(&normalized, "worth_ui") {
                    violations.push(format!(
                        "{} reaches the worth-ui product facade; host adapters must stay on host-contract-only surfaces",
                        file.absolute_path().display()
                    ));
                }
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub(crate) fn normalize_manifest_alias_path(
    segments: &[String],
    manifest_aliases: &HashMap<String, String>,
) -> Vec<String> {
    let mut normalized = segments.to_vec();
    if let Some(first) = normalized.first_mut() {
        if let Some(package_name) = manifest_aliases.get(first) {
            *first = package_name.clone();
        }
    }
    normalized
}
