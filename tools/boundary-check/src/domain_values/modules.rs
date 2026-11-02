//! Resolve through real Rust module declarations and explicit imports.
use crate::config::Files;
use std::collections::BTreeMap;
use syn::Item;

#[derive(Clone)]
pub(super) struct Module {
    pub name: Vec<String>,
    pub source: String,
    directory: String,
    attribute_directory: String,
    pub items: Vec<Item>,
}
pub(super) struct Sources<'a> {
    pub files: &'a Files,
    pub roots: BTreeMap<String, String>,
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
    fn file(&self, name: Vec<String>, source: String, directory: String) -> Result<Module, String> {
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
        Ok(Module {
            name,
            source,
            directory,
            attribute_directory,
            items: file.items,
        })
    }
    fn root(&self, name: &str) -> Result<Module, String> {
        let source = self
            .roots
            .get(name)
            .ok_or_else(|| format!("unknown crate {name}"))?;
        let (parent, _) = source
            .rsplit_once('/')
            .ok_or("crate source has no directory")?;
        self.file(vec![name.into()], source.clone(), parent.into())
    }
    pub(super) fn child(&self, parent: &Module, child: &syn::ItemMod) -> Result<Module, String> {
        super::attributes::check(&child.attrs, &parent.items)?;
        let mut name = parent.name.clone();
        name.push(child.ident.to_string());
        let directory = join(&parent.directory, &child.ident.to_string())?;
        if let Some((_, items)) = &child.content {
            return Ok(Module {
                name,
                source: parent.source.clone(),
                attribute_directory: directory.clone(),
                directory,
                items: items.clone(),
            });
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
                            return self.file(name, source, directory);
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
            (true, false) => self.file(name, flat, directory),
            (false, true) => self.file(name, nested, directory),
            _ => Err(format!("missing or ambiguous module {}", name.join("::"))),
        }
    }
    pub fn module(&self, path: &[String]) -> Result<Module, String> {
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
    pub fn find_source(&self, source: &str) -> Result<Vec<String>, String> {
        // Longest real crate-root directory owns a source, not a caller mapping.
        let mut roots: Vec<_> = self
            .roots
            .iter()
            .filter(|(_, root)| {
                root.rsplit_once('/')
                    .is_some_and(|(directory, _)| source.starts_with(&format!("{directory}/")))
            })
            .collect();
        roots.sort_by_key(|(_, path)| std::cmp::Reverse(path.len()));
        let Some((root, _)) = roots.first() else {
            return Err(format!("no Cargo library owns {source}"));
        };
        let mut found = Vec::new();
        self.find(&self.root(root)?, source, &mut found, 0)?;
        if found.len() != 1 {
            return Err(format!(
                "source needs one actual module context: {source} (found {})",
                found.len()
            ));
        }
        Ok(found.remove(0))
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
