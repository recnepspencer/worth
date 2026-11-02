//! Binding syntax is proof only when it reaches the platform's actual macro.
use super::modules::Sources;
use syn::{Item, Path};

impl Sources<'_> {
    pub(super) fn macro_origin(
        &self,
        context: &[String],
        path: &Path,
        depth: usize,
    ) -> Result<(), String> {
        let mut names: Vec<_> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        if path.leading_colon.is_some() {
            names.insert(0, "::".into());
        }
        self.macro_names(context, &names, depth)
    }
    fn macro_names(&self, context: &[String], path: &[String], depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("macro import resolution depth refused".into());
        }
        let (first, rest) = path.split_first().ok_or("empty macro path")?;
        if first == "::" {
            return self.external_macro(rest);
        }
        if ["crate", "self", "super"].contains(&first.as_str()) {
            let mut owner = context.to_vec();
            match first.as_str() {
                "crate" => owner.truncate(1),
                "super" => {
                    if owner.len() < 2 {
                        return Err("macro path escapes crate".into());
                    }
                    owner.pop();
                }
                _ => (),
            }
            return self.macro_names(&owner, rest, depth + 1);
        }
        let module = self.module(context)?;
        let (imports, glob) = module.imports(first);
        if imports.len() > 1 {
            return Err("ambiguous binding macro import".into());
        }
        if let Some(import) = imports.first() {
            let mut target = import.clone();
            target.extend_from_slice(rest);
            return self.macro_names(context, &target, depth + 1);
        }
        if module.items.iter().any(|item| matches!(item, Item::Macro(item) if item.ident.as_ref().is_some_and(|name| name == first))) {
            return Err("unknown locally generated binding macro".into());
        }
        let children: Vec<_> = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) if module.ident == first => Some(module),
                _ => None,
            })
            .collect();
        if !children.is_empty() {
            if children.len() != 1 {
                return Err("ambiguous binding macro module".into());
            }
            let mut owner = context.to_vec();
            owner.push(first.clone());
            return self.macro_names(&owner, rest, depth + 1);
        }
        if self.roots.contains_key(first) {
            return self.external_macro(path);
        }
        if context == ["worth_query_declaration"] && !glob {
            let mut owned = context.to_vec();
            owned.extend_from_slice(path);
            return self.external_macro(&owned);
        }
        Err(format!("unknown binding macro origin {}", path.join("::")))
    }
    fn external_macro(&self, path: &[String]) -> Result<(), String> {
        let name = path.join("::");
        let known = [
            "worth_query_decl::facade::worth_query_value_binding",
            "worth_query_declaration::worth_query_value_binding",
            "worth_query_declaration::__worth_query_binary_value_binding",
        ];
        if known.contains(&name.as_str())
            && path
                .first()
                .is_some_and(|root| self.roots.contains_key(root))
        {
            return Ok(());
        }
        Err(format!("unknown binding macro origin {name}"))
    }
}
