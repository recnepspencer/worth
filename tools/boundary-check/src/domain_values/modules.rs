//! Resolve through real Rust module declarations and explicit imports.
use crate::config::Files;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use syn::Item;

#[derive(Clone)]
pub(super) struct Module {
    pub name: Vec<String>,
    pub source: String,
    directory: String,
    attribute_directory: String,
    pub items: Vec<Item>,
    included: Vec<String>,
    namespace_ambiguous: bool,
}
pub(super) struct Sources<'a> {
    pub files: &'a Files,
    pub roots: BTreeMap<String, String>,
    pub certified: RefCell<BTreeMap<Vec<String>, Module>>,
    parsed: RefCell<BTreeMap<Vec<String>, Module>>,
    pub syntax: RefCell<BTreeSet<Vec<String>>>,
}

fn join(directory: &str, relative: &str) -> Result<String, String> {
    let mut parts: Vec<_> = directory
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    for part in relative.split('/') {
        match part {
            "." | "" => (),
            ".." => {
                parts.pop().ok_or("module path escapes its source root")?;
            }
            part if part.contains(':') || part.contains('\\') => {
                return Err("unsupported absolute module path".into())
            }
            part => parts.push(part),
        }
    }
    Ok(parts.join("/"))
}
impl Sources<'_> {
    pub(super) fn new(
        files: &crate::config::Files,
        roots: BTreeMap<String, String>,
    ) -> Sources<'_> {
        Sources {
            files,
            roots,
            certified: RefCell::new(BTreeMap::new()),
            parsed: RefCell::new(BTreeMap::new()),
            syntax: RefCell::new(BTreeSet::new()),
        }
    }
    fn file(
        &self,
        name: Vec<String>,
        source: String,
        directory: String,
        namespace_ambiguous: bool,
    ) -> Result<Module, String> {
        if !namespace_ambiguous && let Some(module) = self.parsed.borrow().get(&name) {
            return Ok(module.clone());
        }
        let text = self
            .files
            .get(&source)
            .ok_or_else(|| format!("module source unavailable: {source}"))?;
        let file = syn::parse_file(text).map_err(|error| format!("{source}: {error}"))?;
        super::attributes::check(&file.attrs, &file.items)?;
        let attribute_directory = source
            .rsplit_once('/')
            .ok_or("source directory missing")?
            .0
            .to_owned();
        let mut module = Module {
            name,
            source,
            directory,
            attribute_directory,
            items: file.items,
            included: Vec::new(),
            namespace_ambiguous,
        };
        self.expand_includes(&mut module)?;
        if !namespace_ambiguous {
            self.parsed
                .borrow_mut()
                .insert(module.name.clone(), module.clone());
        }
        Ok(module)
    }
    fn root(&self, name: &str) -> Result<Module, String> {
        let source = self
            .roots
            .get(name)
            .ok_or_else(|| format!("unknown crate {name}"))?;
        let (parent, _) = source
            .rsplit_once('/')
            .ok_or("crate source has no directory")?;
        self.file(vec![name.into()], source.clone(), parent.into(), false)
    }
    pub(super) fn child(&self, parent: &Module, child: &syn::ItemMod) -> Result<Module, String> {
        super::attributes::check(&child.attrs, &parent.items)?;
        let mut name = parent.name.clone();
        name.push(child.ident.to_string());
        let namespace_ambiguous = parent.namespace_ambiguous
            || parent
                .items
                .iter()
                .filter(|item| matches!(item, Item::Mod(module) if module.ident == child.ident))
                .count()
                != 1;
        if !namespace_ambiguous && let Some(module) = self.parsed.borrow().get(&name) {
            return Ok(module.clone());
        }
        let directory = join(&parent.directory, &child.ident.to_string())?;
        if let Some((_, items)) = &child.content {
            let mut module = Module {
                name,
                source: parent.source.clone(),
                attribute_directory: directory.clone(),
                directory,
                items: items.clone(),
                included: Vec::new(),
                namespace_ambiguous,
            };
            self.expand_includes(&mut module)?;
            if !namespace_ambiguous {
                self.parsed
                    .borrow_mut()
                    .insert(module.name.clone(), module.clone());
            }
            return Ok(module);
        }
        for attribute in &child.attrs {
            if attribute.path().is_ident("path") {
                if let syn::Meta::NameValue(value) = &attribute.meta {
                    if let syn::Expr::Lit(value) = &value.value {
                        if let syn::Lit::Str(path) = &value.lit {
                            let source = join(&parent.attribute_directory, &path.value())?;
                            let (parent, filename) =
                                source.rsplit_once('/').ok_or("module directory missing")?;
                            let directory = if filename == "mod.rs" {
                                parent.to_owned()
                            } else {
                                join(
                                    parent,
                                    filename
                                        .strip_suffix(".rs")
                                        .ok_or("module must be Rust source")?,
                                )?
                            };
                            return self.file(name, source, directory, namespace_ambiguous);
                        }
                    }
                }
                return Err("nonliteral module path is unsupported".into());
            }
        }
        let flat = format!("{directory}.rs");
        let nested = format!("{directory}/mod.rs");
        match (
            self.files.contains_key(&flat),
            self.files.contains_key(&nested),
        ) {
            (true, false) => self.file(name, flat, directory, namespace_ambiguous),
            (false, true) => self.file(name, nested, directory, namespace_ambiguous),
            _ => Err(format!("missing or ambiguous module {}", name.join("::"))),
        }
    }
    pub fn module(&self, path: &[String]) -> Result<Module, String> {
        if let Some(module) = self.parsed.borrow().get(path) {
            return Ok(module.clone());
        }
        let (root, rest) = path.split_first().ok_or("empty module path")?;
        let mut module = self.root(root)?;
        for segment in rest {
            let children: Vec<_> = module
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Mod(child) if child.ident == segment => Some(child),
                    _ => None,
                })
                .collect();
            if children.len() != 1 {
                return Err(format!("missing or ambiguous module {segment}"));
            }
            module = self.child(&module, children[0])?;
        }
        Ok(module)
    }
    fn expand_includes(&self, module: &mut Module) -> Result<(), String> {
        let directory = module
            .source
            .rsplit_once('/')
            .ok_or("source directory missing")?
            .0;
        module.items = self.include_items(&module.items, directory, &mut module.included, 0)?;
        Ok(())
    }
    fn include_items(
        &self,
        items: &[Item],
        directory: &str,
        included: &mut Vec<String>,
        depth: usize,
    ) -> Result<Vec<Item>, String> {
        if depth > 128 {
            return Err("source inclusion depth refused".into());
        }
        let mut expanded = Vec::new();
        for item in items {
            if let Item::Macro(item) = item
                && item.mac.path.is_ident("include")
            {
                super::attributes::check(&item.attrs, items)?;
                let path = syn::parse2::<syn::LitStr>(item.mac.tokens.clone())
                    .map_err(|_| "source inclusion requires a literal path")?;
                let path = join(directory, &path.value())?;
                let text = self
                    .files
                    .get(&path)
                    .ok_or_else(|| format!("included source unavailable: {path}"))?;
                let file = syn::parse_file(text).map_err(|error| format!("{path}: {error}"))?;
                super::attributes::check(&file.attrs, &file.items)?;
                if file.items.iter().any(|item| matches!(item, Item::Mod(_))) {
                    return Err(
                        "included module declarations require an explicit module source".into(),
                    );
                }
                included.push(path.clone());
                let directory = path
                    .rsplit_once('/')
                    .ok_or("included source directory missing")?
                    .0;
                expanded.extend(self.include_items(&file.items, directory, included, depth + 1)?);
            } else {
                expanded.push(item.clone());
            }
        }
        Ok(expanded)
    }
    pub fn find_sources(&self, source: &str) -> Result<Vec<Vec<String>>, String> {
        // Longest real crate-root directory owns a source, not a caller mapping.
        let mut roots: Vec<_> = self
            .roots
            .iter()
            .filter(|(_, root)| {
                root.rsplit_once('/')
                    .is_some_and(|(directory, _)| source.starts_with(&format!("{directory}/")))
            })
            .collect();
        roots.sort_by_key(|(_, path)| std::cmp::Reverse(path.rsplit_once('/').unwrap().0.len()));
        let Some((_, nearest)) = roots.first() else {
            return Err(format!("no Cargo crate owns {source}"));
        };
        let directory = nearest.rsplit_once('/').unwrap().0;
        let mut found = Vec::new();
        for (root, path) in roots {
            if path.rsplit_once('/').unwrap().0 == directory {
                self.find(&self.root(root)?, source, &mut found, 0)?;
            }
        }
        if found.is_empty() {
            return Err(format!(
                "source needs an actual module context: {source} (found {})",
                found.len()
            ));
        }
        Ok(found)
    }
    fn find(
        &self,
        module: &Module,
        source: &str,
        found: &mut Vec<Vec<String>>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 128 {
            return Err("module traversal depth refused".into());
        }
        if module.source == source {
            found.push(module.name.clone());
            return Ok(());
        }
        if module.included.iter().any(|path| path == source) {
            found.push(module.name.clone());
        }
        for item in &module.items {
            if let Item::Mod(child) = item {
                // A descendant #[path] can redirect outside its parent's
                // default directory. Walk actual declarations, never infer
                // ownership from the sought file's spelling.
                self.find(&self.child(module, child)?, source, found, depth + 1)?;
            }
        }
        Ok(())
    }
}
