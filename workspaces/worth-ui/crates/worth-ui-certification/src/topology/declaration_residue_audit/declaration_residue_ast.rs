use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{ExprMethodCall, File, ImplItem, Item, ItemExternCrate, ItemUse, UseTree};

use super::{
    declaration_semantic_authority_path, should_skip_non_owner_audit_file,
    DECLARATION_SEMANTIC_TOKEN_MARKERS, DECLARATION_SOURCE_REOPENING_ALLOWED_FILES,
    DECLARATION_SOURCE_REOPENING_METHODS,
};
use crate::topology::dependency_audit::path_starts_with;
use crate::topology::workspace_source_inventory::WorkspaceSourceInventory;

pub(super) fn starts_with_declaration_surface(segments: &[String]) -> bool {
    (path_starts_with(segments, "worth_ui")
        && segments.get(1).is_some_and(|segment| segment == "facade")
        && segments
            .get(2)
            .is_some_and(|segment| segment == "declaration"))
        || (path_starts_with(segments, "worth_ui_runtime")
            && ((segments
                .get(1)
                .is_some_and(|segment| segment == "declaration"))
                || (segments.get(1).is_some_and(|segment| segment == "facade")
                    && segments
                        .get(2)
                        .is_some_and(|segment| segment == "declaration"))))
}

pub(super) fn collect_method_names(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<String> {
    let parsed = parse_rust_file(inventory, path);
    let mut collector = MethodCallCollector::default();
    collector.visit_file(&parsed);
    collector.method_names
}

pub(super) fn collect_method_names_for_function(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
    function_name: &str,
) -> Vec<String> {
    let parsed = parse_rust_file(inventory, path);
    let mut collector = MethodCallCollector::default();

    for item in parsed.items {
        match item {
            Item::Fn(item_fn) if item_fn.sig.ident == function_name => {
                collector.visit_block(&item_fn.block);
            }
            Item::Impl(item_impl) => {
                for impl_item in item_impl.items {
                    if let ImplItem::Fn(function) = impl_item
                        && function.sig.ident == function_name
                    {
                        collector.visit_block(&function.block);
                    }
                }
            }
            _ => {}
        }
    }

    collector.method_names
}

pub(super) fn production_source_text(inventory: &WorkspaceSourceInventory, path: &Path) -> String {
    let text = inventory.text(path);
    if let Some(cfg_test_start) = text.find("#[cfg(test)]") {
        text[..cfg_test_start].to_string()
    } else {
        text.to_owned()
    }
}

pub(super) fn collect_file_paths(
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

pub(super) fn collect_file_use_paths(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<Vec<String>> {
    let parsed = parse_rust_file(inventory, path);
    let mut collector = UsePathCollector::default();
    collector.visit_file(&parsed);
    collector.collected_paths
}

pub(super) fn audit_files_do_not_reopen_declaration_source(
    inventory: &WorkspaceSourceInventory,
    files: &[PathBuf],
) -> Vec<String> {
    let mut violations = Vec::new();

    for path in files {
        let relative = path
            .strip_prefix(inventory.root())
            .expect("workspace file should strip to relative path");
        let relative_text = relative.to_string_lossy().replace('\\', "/");

        if should_skip_non_owner_audit_file(&relative_text) {
            continue;
        }

        if DECLARATION_SOURCE_REOPENING_ALLOWED_FILES
            .iter()
            .any(|allowed| *allowed == relative_text)
        {
            continue;
        }

        for segments in collect_file_paths(inventory, path)
            .into_iter()
            .chain(collect_file_use_paths(inventory, path))
        {
            if let Some(authority_name) = declaration_semantic_authority_path(&segments) {
                violations.push(format!(
                    "{} reopens declaration meaning by reaching DSL semantic authority type `{authority_name}` outside the owning declaration lowering/admission lanes",
                    path.display()
                ));
            }
        }

        for method_name in collect_method_names(inventory, path) {
            if DECLARATION_SOURCE_REOPENING_METHODS.contains(&method_name.as_str()) {
                violations.push(format!(
                    "{} reopens declaration meaning through DSL semantic accessor `{method_name}()` outside the owning declaration lowering/admission lanes",
                    path.display()
                ));
            }
        }

        let source = production_source_text(inventory, path);
        for marker in DECLARATION_SEMANTIC_TOKEN_MARKERS {
            if source.contains(marker) {
                violations.push(format!(
                    "{} reinterprets declaration semantics through raw declaration token vocabulary `{marker}` outside the owning declaration lowering/admission lanes",
                    path.display()
                ));
            }
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

fn parse_rust_file(inventory: &WorkspaceSourceInventory, path: &Path) -> File {
    let text = source_without_test_module_tail(inventory, path);
    syn::parse_file(&text).unwrap_or_else(|error| {
        panic!("{} should parse as Rust source: {error}", path.display());
    })
}

fn source_without_test_module_tail(inventory: &WorkspaceSourceInventory, path: &Path) -> String {
    let text = inventory.text(path);
    if let Some(test_module_start) = text
        .find("#[cfg(test)]\nmod tests")
        .or_else(|| text.find("#[cfg(test)]\r\nmod tests"))
    {
        text[..test_module_start].to_string()
    } else {
        text.to_owned()
    }
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

impl Visit<'_> for PathCollector<'_> {
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

#[derive(Default)]
struct MethodCallCollector {
    method_names: Vec<String>,
}

impl Visit<'_> for MethodCallCollector {
    fn visit_expr_method_call(&mut self, method_call: &ExprMethodCall) {
        self.method_names.push(method_call.method.to_string());
        visit::visit_expr_method_call(self, method_call);
    }
}
