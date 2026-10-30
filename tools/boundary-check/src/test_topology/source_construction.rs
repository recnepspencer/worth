//! Syntactic construction evidence and local-fixture propagation for source tests.
use super::{category_paths::test_attribute, runtime_roots::Registry};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use syn::visit::Visit;

mod enum_variants;
mod expressions;
mod imports;
use expressions::Body;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(super) struct Construction {
    pub source: String,
    pub test: String,
    pub constructor: String,
}

#[derive(Default)]
struct Function {
    source: String,
    name: String,
    test: bool,
    calls: BTreeSet<String>,
    constructors: BTreeSet<String>,
}

pub(super) fn inventory(
    root: &Path,
    source: &Path,
    roots: &Registry,
) -> Result<BTreeSet<Construction>, String> {
    let manifest =
        crate::source_documents::read_to_string(&source.parent().unwrap().join("Cargo.toml"))
            .map_err(|e| e.to_string())?;
    let manifest: toml::Value = toml::from_str(&manifest).map_err(|e| e.to_string())?;
    let mut foreign = BTreeSet::new();
    let mut tables = vec![&manifest];
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        tables.extend(targets.values());
    }
    for table in tables {
        for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = table.get(kind).and_then(toml::Value::as_table) {
                foreign.extend(deps.keys().map(|n| n.replace('-', "_")));
            }
        }
    }
    let owner = crate::source_rules::GovernedCrate {
        package: manifest["package"]["name"]
            .as_str()
            .ok_or("missing package name")?
            .into(),
        crate_root: source.parent().unwrap().into(),
        relative_crate_root: source
            .parent()
            .unwrap()
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/"),
    };
    foreign.remove(&owner.package.replace('-', "_"));
    let mut functions = Vec::new();
    let mut targets = Vec::new();
    for name in ["lib.rs", "main.rs"] {
        let path = source.join(name);
        if path.exists() {
            targets.push(path);
        }
    }
    if let Some(path) = manifest
        .get("lib")
        .and_then(|v| v.get("path"))
        .and_then(toml::Value::as_str)
    {
        targets.push(owner.crate_root.join(path));
    }
    for bin in manifest
        .get("bin")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(path) = bin.get("path").and_then(toml::Value::as_str) {
            targets.push(owner.crate_root.join(path));
        }
    }
    if source.join("bin").is_dir() {
        for entry in std::fs::read_dir(source.join("bin")).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "rs") {
                targets.push(path);
            } else if path.join("main.rs").is_file() {
                targets.push(path.join("main.rs"));
            }
        }
    }
    targets.sort();
    targets.dedup();
    for target in targets {
        let graph = crate::source_rules::parse_compiled_target_where(&owner, &target, &|_| true)?;
        let scopes = imports::graph_bindings(&graph, &foreign);
        let enum_variants = enum_variants::owners(&graph);
        for (scope, node) in graph.modules {
            let physical = std::fs::canonicalize(
                owner
                    .crate_root
                    .join(node.relative_source.split(';').next().unwrap()),
            )
            .map_err(|e| e.to_string())?;
            let relative = physical
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if !physical.starts_with(source)
                && physical.components().any(|p| p.as_os_str() == "tests")
            {
                return Err(format!(
                    "product src cannot mount a tests/ module: {}",
                    physical.display()
                ));
            }
            let inherited = scopes.get(&scope).cloned().unwrap_or_default();
            let mut collector = Functions {
                source: relative,
                scope: scope.clone(),
                imports: inherited,
                enum_variants: &enum_variants,
                foreign: &foreign,
                roots,
                package: &owner.package.replace('-', "_"),
                functions: &mut functions,
            };
            // The module graph already expanded each inline/file module exactly once.
            let items: Vec<_> = node
                .items
                .into_iter()
                .filter(|i| !matches!(i, syn::Item::Mod(_)))
                .collect();
            collector.items(&items);
        }
    }
    // A source court often constructs its runtime through a local fixture.
    // Iterate to a fixed point rather than treating a helper call as innocence.
    loop {
        let mut by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for function in &functions {
            by_name
                .entry(function.name.clone())
                .or_default()
                .extend(function.constructors.iter().cloned());
        }
        let mut changed = false;
        for function in &mut functions {
            let before = function.constructors.len();
            for call in &function.calls {
                let lexical = call.starts_with('?');
                let mut candidate = call.trim_start_matches('?').to_owned();
                loop {
                    if let Some(constructors) = by_name.get(&candidate) {
                        function.constructors.extend(constructors.iter().cloned());
                        break;
                    }
                    let mut path: Vec<_> = candidate.split("::").collect();
                    if !lexical || path.len() < 2 {
                        break;
                    }
                    path.remove(path.len() - 2);
                    candidate = path.join("::");
                }
            }
            changed |= before != function.constructors.len();
        }
        if !changed {
            break;
        }
    }
    Ok(functions
        .into_iter()
        .filter(|f| f.test)
        .flat_map(|f| {
            f.constructors
                .into_iter()
                .map(move |constructor| Construction {
                    source: f.source.clone(),
                    test: f.name.clone(),
                    constructor,
                })
        })
        .collect())
}

struct Functions<'a> {
    source: String,
    scope: Vec<String>,
    imports: BTreeMap<String, String>,
    enum_variants: &'a BTreeMap<String, String>,
    foreign: &'a BTreeSet<String>,
    roots: &'a Registry,
    package: &'a str,
    functions: &'a mut Vec<Function>,
}
impl Functions<'_> {
    fn items(&mut self, items: &[syn::Item]) {
        for item in items {
            if let syn::Item::Use(item) = item {
                imports::scoped(&item.tree, &self.scope, self.foreign, &mut self.imports);
            }
        }
        for item in items {
            let name = match item {
                syn::Item::Struct(i) => Some(i.ident.to_string()),
                syn::Item::Enum(i) => Some(i.ident.to_string()),
                _ => None,
            };
            if let Some(name) = name {
                self.imports.insert(
                    name.clone(),
                    std::iter::once("crate".to_owned())
                        .chain(self.scope.iter().cloned())
                        .chain([name.clone()])
                        .collect::<Vec<_>>()
                        .join("::"),
                );
            }
        }
        for item in items {
            self.visit_item(item);
        }
    }
}
impl Functions<'_> {
    fn record(&mut self, function: &syn::ItemFn) {
        let mut imports = self.imports.clone();
        for statement in &function.block.stmts {
            if let syn::Stmt::Item(syn::Item::Use(item)) = statement {
                let mut local = BTreeMap::new();
                imports::scoped(&item.tree, &self.scope, self.foreign, &mut local);
                imports.extend(local);
            }
        }
        let mut body = Body {
            roots: self.roots,
            package: self.package,
            variables: BTreeMap::new(),
            imports: &imports,
            enum_variants: self.enum_variants,
            foreign: self.foreign,
            scope: &self.scope,
            calls: BTreeSet::new(),
            constructors: BTreeSet::new(),
        };
        body.visit_block(&function.block);
        if function.attrs.iter().any(|attribute| {
            attribute
                .path()
                .segments
                .first()
                .is_some_and(|p| p.ident == "tokio")
                && attribute
                    .path()
                    .segments
                    .last()
                    .is_some_and(|p| p.ident == "test")
        }) && self.roots.matching("tokio::runtime::Runtime").is_some()
        {
            body.constructors
                .insert("tokio::runtime::Runtime::<test-harness>".into());
        }
        self.functions.push(Function {
            source: self.source.clone(),
            name: self
                .scope
                .iter()
                .cloned()
                .chain([function.sig.ident.to_string()])
                .collect::<Vec<_>>()
                .join("::"),
            test: function.attrs.iter().any(|a| test_attribute(&a.meta)),
            calls: body.calls,
            constructors: body.constructors,
        });
    }
}
impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if let Some((_, items)) = &module.content {
            let inherited = self.imports.clone();
            self.scope.push(module.ident.to_string());
            self.items(items);
            self.scope.pop();
            self.imports = inherited;
        }
    }
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.record(function);
    }
    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        let syn::Type::Path(ty) = &*implementation.self_ty else {
            return;
        };
        let Some(owner) = ty.path.segments.last() else {
            return;
        };
        let inherited = self.imports.clone();
        let identity = self
            .imports
            .get(&owner.ident.to_string())
            .cloned()
            .unwrap_or_else(|| {
                std::iter::once("crate".to_owned())
                    .chain(self.scope.iter().cloned())
                    .chain([owner.ident.to_string()])
                    .collect::<Vec<_>>()
                    .join("::")
            });
        self.imports.insert("Self".into(), identity);
        self.scope.push(owner.ident.to_string());
        for item in &implementation.items {
            if let syn::ImplItem::Fn(method) = item {
                let function = syn::ItemFn {
                    attrs: method.attrs.clone(),
                    vis: method.vis.clone(),
                    sig: method.sig.clone(),
                    block: Box::new(method.block.clone()),
                };
                self.record(&function);
            }
        }
        self.scope.pop();
        self.imports = inherited;
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if let Ok(file) = syn::parse2::<syn::File>(item.mac.tokens.clone()) {
            self.items(&file.items);
        } else if self.source.contains("/tests/")
            || self.source.ends_with("tests.rs")
            || self.scope.iter().any(|p| p == "tests")
        {
            // Keep macro-family evidence only when its arguments construct a
            // registered root or call a local fixture that does so.
            let mut body = Body {
                package: self.package,
                variables: BTreeMap::new(),
                roots: self.roots,
                imports: &self.imports,
                enum_variants: self.enum_variants,
                foreign: self.foreign,
                scope: &self.scope,
                calls: BTreeSet::new(),
                constructors: BTreeSet::new(),
            };
            body.visit_macro(&item.mac);
            let generator = item
                .mac
                .path
                .segments
                .iter()
                .map(|p| p.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            self.functions.push(Function {
                source: self.source.clone(),
                name: format!(
                    "{}::<generator:{generator}({})>",
                    self.scope.join("::"),
                    item.mac.tokens
                ),
                test: true,
                calls: body.calls,
                constructors: body.constructors,
            });
        }
    }
}
