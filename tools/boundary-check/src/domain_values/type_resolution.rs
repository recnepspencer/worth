//! Actual Rust type/import lookup, shared by closure and macro provenance.
use super::modules::{Module, Sources};
use syn::{Item, Type, UseTree};

pub(super) enum Definition {
    Item(Box<Item>, Vec<String>),
    Builtin(String),
}
impl Module {
    pub(super) fn imports(&self, name: &str) -> (Vec<Vec<String>>, bool) {
        let mut imports = Vec::new();
        let mut glob = false;
        for item in &self.items {
            if let Item::Use(value) = item {
                let prefix = if value.leading_colon.is_some() {
                    vec!["::".into()]
                } else {
                    Vec::new()
                };
                uses(&value.tree, prefix, name, &mut imports, &mut glob);
            }
        }
        (imports, glob)
    }
}

impl Sources<'_> {
    pub(super) fn certify_context(&self, context: &[String]) -> Result<Module, String> {
        let module = self.module(context)?;
        super::attributes::module(&module.items)?;
        for item in &module.items {
            if let Item::Macro(item) = item {
                if !item.mac.path.is_ident("macro_rules") {
                    self.macro_origin(context, &item.mac.path, 0)?;
                }
            }
        }
        Ok(module)
    }
    pub fn resolve(
        &self,
        context: &[String],
        path: &[String],
        depth: usize,
    ) -> Result<Definition, String> {
        if depth > 128 {
            return Err("import resolution depth refused".into());
        }
        let module = self.certify_context(context)?;
        if path.first().is_some_and(|name| name == "::") {
            let external = &path[1..];
            let (root, rest) = external.split_first().ok_or("absolute path has no crate")?;
            if self.roots.contains_key(root) {
                return self.resolve(&[root.clone()], rest, depth + 1);
            }
            let name = external.join("::");
            if builtin_name(&name).is_some() {
                return Ok(Definition::Builtin(name));
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
            return self.resolve(&module, rest, depth + 1);
        }
        let mut definitions = Vec::new();
        let (imports, glob) = module.imports(first);
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
                return self.resolve(&child.name, rest, depth + 1);
            }
            if !rest.is_empty() {
                return Err("associated type paths are unsupported".into());
            }
            return Ok(Definition::Item(
                Box::new((*item).clone()),
                context.to_vec(),
            ));
        }
        if let Some(import) = imports.first() {
            let mut target = import.clone();
            target.extend_from_slice(rest);
            return self.resolve(context, &target, depth + 1);
        }
        if self.roots.contains_key(first) {
            return self.resolve(&[first.clone()], rest, depth + 1);
        }
        let builtin = path.join("::");
        if !glob && builtin_name(&builtin).is_some() {
            return Ok(Definition::Builtin(builtin));
        }
        Err(format!(
            "unresolved type/import {} from {}",
            builtin,
            context.join("::")
        ))
    }
}

fn uses(
    tree: &UseTree,
    prefix: Vec<String>,
    name: &str,
    found: &mut Vec<Vec<String>>,
    glob: &mut bool,
) {
    match tree {
        UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            uses(&path.tree, prefix, name, found, glob);
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
                uses(tree, prefix.clone(), name, found, glob);
            }
        }
        UseTree::Glob(_) => *glob = true,
        _ => (),
    }
}

pub(super) fn builtin_name(path: &str) -> Option<&str> {
    match path {
        "Vec" | "std::vec::Vec" | "alloc::vec::Vec" => Some("Vec"),
        "Box" | "std::boxed::Box" | "alloc::boxed::Box" => Some("Box"),
        "Option" | "std::option::Option" | "core::option::Option" => Some("Option"),
        "String" | "std::string::String" | "alloc::string::String" => Some("String"),
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
