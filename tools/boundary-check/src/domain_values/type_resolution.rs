//! Actual Rust type/import lookup, shared by closure and macro provenance.
use super::modules::{Module, Sources};
use std::collections::BTreeSet;
use syn::{Item, Type, UseTree};

#[derive(Clone)]
pub(super) enum Definition {
    Item(Box<Item>, Vec<String>),
    Builtin(String),
    Module(Module),
}
#[derive(Default)]
pub(super) struct Lookup {
    active: BTreeSet<(Vec<String>, Vec<String>, bool)>,
    pub contexts: BTreeSet<Vec<String>>,
}
impl Definition {
    fn identity(&self) -> String {
        match self {
            Self::Builtin(name) => format!("builtin:{}", builtin_name(name).unwrap()),
            Self::Module(module) => format!("module:{}", module.name.join("::")),
            Self::Item(item, owner) => {
                let name = match item.as_ref() {
                    Item::Struct(value) => &value.ident,
                    Item::Enum(value) => &value.ident,
                    Item::Type(value) => &value.ident,
                    _ => unreachable!("only named type definitions are resolved"),
                };
                format!("{}::{name}", owner.join("::"))
            }
        }
    }
}
impl Module {
    pub(super) fn imports(&self, name: &str) -> (Vec<Vec<String>>, bool) {
        let (named, globs) = self.import_paths(name);
        (named, !globs.is_empty())
    }
    fn import_paths(&self, name: &str) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let mut imports = Vec::new();
        let mut globs = Vec::new();
        for item in &self.items {
            if let Item::Use(value) = item {
                let prefix = if value.leading_colon.is_some() {
                    vec!["::".into()]
                } else {
                    Vec::new()
                };
                uses(&value.tree, prefix, name, &mut imports, &mut globs);
            }
        }
        (imports, globs)
    }
}

impl Sources<'_> {
    pub(super) fn source_context(&self, context: &[String]) -> Result<Module, String> {
        if let Some(module) = self.certified.borrow().get(context) {
            return Ok(module.clone());
        }
        let module = self.module(context)?;
        if self.syntax.borrow().contains(context) {
            return Ok(module);
        }
        super::attributes::module(&module.items)?;
        for item in &module.items {
            if let Item::Macro(item) = item
                && !item.mac.path.is_ident("macro_rules")
            {
                self.macro_origin(context, &item.mac.path, 0)?;
            }
        }
        self.syntax.borrow_mut().insert(context.to_vec());
        Ok(module)
    }
    pub fn resolve(
        &self,
        context: &[String],
        path: &[String],
        depth: usize,
    ) -> Result<Definition, String> {
        let mut lookup = Lookup::default();
        let result = self
            .lookup(context, path, depth, true, &mut lookup)?
            .ok_or_else(|| {
                format!(
                    "unresolved type/import {} from {}",
                    path.join("::"),
                    context.join("::")
                )
            })?;
        for context in lookup.contexts {
            self.certify_context(&context)?;
        }
        Ok(result)
    }
    fn lookup(
        &self,
        context: &[String],
        path: &[String],
        depth: usize,
        prelude: bool,
        active: &mut Lookup,
    ) -> Result<Option<Definition>, String> {
        if depth > 128 {
            return Err("import resolution depth refused".into());
        }
        let key = (context.to_vec(), path.to_vec(), prelude);
        if !active.active.insert(key.clone()) {
            // Cyclic glob re-exports contribute no new candidate on this edge.
            return Ok(None);
        }
        let result = self.lookup_path(context, path, depth, prelude, active);
        active.active.remove(&key);
        result
    }
    fn lookup_path(
        &self,
        context: &[String],
        path: &[String],
        depth: usize,
        prelude: bool,
        active: &mut Lookup,
    ) -> Result<Option<Definition>, String> {
        let module = self.source_context(context)?;
        active.contexts.insert(context.to_vec());
        if path.is_empty() {
            return Ok(Some(Definition::Module(module)));
        }
        if path.first().is_some_and(|name| name == "::") {
            let external = &path[1..];
            let (root, rest) = external.split_first().ok_or("absolute path has no crate")?;
            if self.roots.contains_key(root) {
                return self.lookup(std::slice::from_ref(root), rest, depth + 1, false, active);
            }
            let name = external.join("::");
            if builtin_name(&name).is_some() {
                return Ok(Some(Definition::Builtin(name)));
            }
            return Err(format!("unresolved absolute crate/type {name}"));
        }
        let (first, rest) = path.split_first().ok_or("empty type path")?;
        if ["crate", "self", "super"].contains(&first.as_str()) {
            let mut module = context.to_vec();
            match first.as_str() {
                "crate" => module.truncate(1),
                "super" => {
                    if module.len() < 2 {
                        return Err("super escapes crate".into());
                    }
                    module.pop();
                }
                _ => (),
            }
            return self.lookup(&module, rest, depth + 1, false, active);
        }
        let mut definitions = Vec::new();
        let (imports, _) = module.import_paths(first);
        for item in &module.items {
            match item {
                Item::Struct(value) if value.ident == first => definitions.push(item),
                Item::Enum(value) if value.ident == first => definitions.push(item),
                Item::Type(value) if value.ident == first => definitions.push(item),
                Item::Mod(value) if value.ident == first => definitions.push(item),
                _ => (),
            }
        }
        if definitions.len() + imports.len() > 1 {
            return Err(format!("ambiguous type/import {first}"));
        }
        if let Some(item) = definitions.first() {
            if let Item::Mod(child) = item {
                let child = self.child(&module, child)?;
                return self.lookup(&child.name, rest, depth + 1, false, active);
            }
            if !rest.is_empty() {
                return Err("associated type paths are unsupported".into());
            }
            return Ok(Some(Definition::Item(
                Box::new((*item).clone()),
                context.to_vec(),
            )));
        }
        if let Some(import) = imports.first() {
            let mut target = import.clone();
            target.extend_from_slice(rest);
            return self
                .lookup(context, &target, depth + 1, true, active)?
                .map(Some)
                .ok_or_else(|| format!("unresolved explicit import {}", target.join("::")));
        }
        if self.roots.contains_key(first) && (prelude || !rest.is_empty()) {
            return self.lookup(std::slice::from_ref(first), rest, depth + 1, false, active);
        }
        let candidate = self.glob_candidate(&module, path, depth, active)?;
        if candidate.is_some() {
            return Ok(candidate);
        }
        let builtin = path.join("::");
        if (prelude || path.len() > 1) && builtin_name(&builtin).is_some() {
            return Ok(Some(Definition::Builtin(builtin)));
        }
        Ok(None)
    }
    pub(super) fn glob_candidate(
        &self,
        module: &Module,
        path: &[String],
        depth: usize,
        active: &mut Lookup,
    ) -> Result<Option<Definition>, String> {
        let context = &module.name;
        let (_, globs) = module.import_paths(path.first().ok_or("empty glob lookup")?);
        let mut candidate: Option<Definition> = None;
        for glob in globs {
            if active
                .active
                .contains(&(context.clone(), glob.clone(), true))
            {
                // Resolve a namespace's other import edges before applying its
                // extern prelude fallback; this edge is already in progress.
                continue;
            }
            let namespace = self.lookup(context, &glob, depth + 1, true, active)?;
            let Some(Definition::Module(namespace)) = namespace else {
                return Err(format!(
                    "unresolved/unsupported glob namespace {}",
                    glob.join("::")
                ));
            };
            if let Some(found) = self.lookup(&namespace.name, path, depth + 1, false, active)? {
                if candidate
                    .as_ref()
                    .is_some_and(|previous| previous.identity() != found.identity())
                {
                    return Err(format!("ambiguous glob type/import {}", path.join("::")));
                }
                candidate = Some(found);
            }
        }
        Ok(candidate)
    }
}

fn uses(
    tree: &UseTree,
    prefix: Vec<String>,
    name: &str,
    found: &mut Vec<Vec<String>>,
    globs: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            uses(&path.tree, prefix, name, found, globs);
        }
        UseTree::Name(value) if value.ident == name => {
            let mut prefix = prefix;
            prefix.push(value.ident.to_string());
            found.push(prefix);
        }
        UseTree::Rename(value) if value.rename == name => {
            let mut prefix = prefix;
            prefix.push(value.ident.to_string());
            found.push(prefix);
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                uses(tree, prefix.clone(), name, found, globs);
            }
        }
        UseTree::Glob(_) => globs.push(prefix),
        _ => (),
    }
}

pub(super) fn builtin_name(path: &str) -> Option<&str> {
    match path {
        "Vec" | "std::vec::Vec" | "alloc::vec::Vec" => Some("Vec"),
        "Box" | "std::boxed::Box" | "alloc::boxed::Box" => Some("Box"),
        "Option" | "std::option::Option" | "core::option::Option" => Some("Option"),
        "String" | "std::string::String" | "alloc::string::String" => Some("String"),
        "std::marker::PhantomData" | "core::marker::PhantomData" => Some("PhantomData"),
        "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
        | "i128" | "isize" | "f32" | "f64" | "char" => Some(path),
        _ => None,
    }
}

pub(super) fn type_path(ty: &Type) -> Result<Vec<String>, String> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let mut names = Vec::new();
            if path.path.leading_colon.is_some() {
                names.push("::".into());
            }
            names.extend(
                path.path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string()),
            );
            Ok(names)
        }
        _ => Err("binding/type must have a concrete owned path".into()),
    }
}
