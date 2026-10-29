//! Expand named foreign imports without treating a type reference as construction.
use crate::source_rules::ModuleGraph;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    imports: &mut BTreeMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect(&path.tree, prefix, imports);
            prefix.pop();
        }
        syn::UseTree::Name(name) => {
            let name = name.ident.to_string();
            let expanded = prefix
                .iter()
                .cloned()
                .chain([name.clone()])
                .collect::<Vec<_>>()
                .join("::");
            if !prefix.is_empty() {
                imports.insert(name, expanded);
            }
        }
        syn::UseTree::Rename(rename) => {
            let expanded = prefix
                .iter()
                .cloned()
                .chain([rename.ident.to_string()])
                .collect::<Vec<_>>()
                .join("::");
            if !prefix.is_empty() {
                imports.insert(rename.rename.to_string(), expanded);
            }
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect(item, prefix, imports);
            }
        }
        syn::UseTree::Glob(_) => {
            if !prefix.is_empty() {
                let root = prefix.join("::");
                imports.insert(format!("*foreign:{root}"), root);
            }
        }
    }
}
pub(super) fn scoped(
    tree: &syn::UseTree,
    scope: &[String],
    foreign: &BTreeSet<String>,
    into: &mut BTreeMap<String, String>,
) {
    let mut raw = BTreeMap::new();
    collect(tree, &mut Vec::new(), &mut raw);
    for (name, path) in raw {
        let absolute = absolute(scope, &path, foreign);
        if name.starts_with("*foreign:") {
            if foreign.contains(absolute.split("::").next().unwrap()) {
                into.insert(name, absolute);
            }
        } else {
            into.entry(name).or_insert(absolute);
        }
    }
}
fn absolute(scope: &[String], path: &str, foreign: &BTreeSet<String>) -> String {
    let mut parts: Vec<_> = path.split("::").map(str::to_owned).collect();
    if parts.first().is_some_and(|p| {
        p == "crate" || foreign.contains(p) || ["std", "core", "alloc"].contains(&p.as_str())
    }) {
        return path.into();
    }
    let mut module = scope.to_vec();
    while parts.first().is_some_and(|p| p == "super" || p == "self") {
        if parts.remove(0) == "super" {
            module.pop();
        }
    }
    std::iter::once("crate".into())
        .chain(module)
        .chain(parts)
        .collect::<Vec<_>>()
        .join("::")
}
pub(super) fn graph_bindings(
    graph: &ModuleGraph,
    foreign: &BTreeSet<String>,
) -> BTreeMap<Vec<String>, BTreeMap<String, String>> {
    let mut scopes: BTreeMap<Vec<String>, BTreeMap<String, String>> = BTreeMap::new();
    let mut globs = Vec::new();
    for (scope, node) in &graph.modules {
        let mut names = BTreeMap::new();
        for item in &node.items {
            let name = match item {
                syn::Item::Struct(i) => Some(i.ident.to_string()),
                syn::Item::Enum(i) => Some(i.ident.to_string()),
                syn::Item::Type(i) => Some(i.ident.to_string()),
                syn::Item::Fn(i) => Some(i.sig.ident.to_string()),
                syn::Item::Mod(i) => Some(i.ident.to_string()),
                _ => None,
            };
            if let Some(name) = name {
                names.insert(
                    name.clone(),
                    std::iter::once("crate".into())
                        .chain(scope.iter().cloned())
                        .chain([name])
                        .collect::<Vec<_>>()
                        .join("::"),
                );
            }
            if let syn::Item::Type(alias) = item {
                if let syn::Type::Path(ty) = &*alias.ty {
                    let path = ty
                        .path
                        .segments
                        .iter()
                        .map(|p| p.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    names.insert(alias.ident.to_string(), absolute(scope, &path, foreign));
                }
            }
            if let syn::Item::Use(i) = item {
                let mut raw = BTreeMap::new();
                collect(&i.tree, &mut Vec::new(), &mut raw);
                for (name, path) in raw {
                    let path = absolute(scope, &path, foreign);
                    if name.starts_with("*foreign:") && path.starts_with("crate::") {
                        globs.push((
                            scope.clone(),
                            path.trim_start_matches("crate::")
                                .split("::")
                                .map(str::to_owned)
                                .collect::<Vec<_>>(),
                        ));
                    } else {
                        names.insert(name, path);
                    }
                }
            }
        }
        scopes.insert(scope.clone(), names);
    }
    // Only finite name insertion uses a fixed point. Alias expansion always
    // starts from immutable declarations, with a visited-symbol stack.
    loop {
        let before = scopes.clone();
        for (scope, target) in &globs {
            if let Some(names) = before.get(target) {
                for (name, value) in names {
                    scopes
                        .get_mut(scope)
                        .unwrap()
                        .entry(name.clone())
                        .or_insert_with(|| value.clone());
                }
            }
        }
        if scopes == before {
            break;
        }
    }
    let declared = scopes.clone();
    for names in scopes.values_mut() {
        for value in names.values_mut() {
            *value = resolve(value, &declared, &mut BTreeSet::new());
        }
    }
    scopes
}

fn resolve(
    path: &str,
    scopes: &BTreeMap<Vec<String>, BTreeMap<String, String>>,
    seen: &mut BTreeSet<(Vec<String>, String)>,
) -> String {
    if !path.starts_with("crate::") {
        return path.into();
    }
    let parts: Vec<_> = path
        .trim_start_matches("crate::")
        .split("::")
        .map(str::to_owned)
        .collect();
    for end in (1..=parts.len()).rev() {
        let module = parts[..end - 1].to_vec();
        let Some(names) = scopes.get(&module) else {
            continue;
        };
        let Some(binding) = names.get(&parts[end - 1]) else {
            continue;
        };
        let prefix = format!("crate::{}", parts[..end].join("::"));
        if binding == &prefix {
            continue;
        }
        if !seen.insert((module, parts[end - 1].clone())) {
            return format!("unresolved::{path}");
        }
        let expanded = std::iter::once(binding.as_str())
            .chain(parts[end..].iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("::");
        return resolve(&expanded, scopes, seen);
    }
    path.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn three_alias_cycles_terminate_as_unresolved() {
        let names = BTreeMap::from([
            ("A".into(), "crate::B".into()),
            ("B".into(), "crate::C".into()),
            ("C".into(), "crate::A".into()),
        ]);
        let scopes = BTreeMap::from([(Vec::new(), names)]);
        assert!(resolve("crate::A", &scopes, &mut BTreeSet::new()).starts_with("unresolved::"));
    }
    #[test]
    fn recursive_alias_suffixes_terminate_without_path_growth() {
        let scopes = BTreeMap::from([(
            Vec::new(),
            BTreeMap::from([("A".into(), "crate::A::Tail".into())]),
        )]);
        let result = resolve("crate::A", &scopes, &mut BTreeSet::new());
        assert_eq!(result, "unresolved::crate::A::Tail");
    }
    #[test]
    fn canonical_local_bindings_and_foreign_alias_chains_keep_their_owner() {
        let scopes = BTreeMap::from([(
            Vec::new(),
            BTreeMap::from([
                ("Local".into(), "crate::Local".into()),
                ("Alias".into(), "foreign::Value".into()),
            ]),
        )]);
        assert_eq!(
            resolve("crate::Local::new", &scopes, &mut BTreeSet::new()),
            "crate::Local::new"
        );
        assert_eq!(
            resolve("crate::Alias::new", &scopes, &mut BTreeSet::new()),
            "foreign::Value::new"
        );
    }
}
