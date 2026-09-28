//! One module's name table: explicit items and imports shadow glob imports;
//! crate-root `#[macro_export]` macros and extern crates fill in after them.

use super::item_shape::{defined_name, is_test_only, macro_rules_name};
use super::use_bindings::{use_bindings, BoundName, UseBinding};
use super::{module_definition, CrateId, Definition, LookupKey, Reach, Resolver};
use syn::{Item, Visibility};

impl Resolver<'_> {
    /// Every definition `name` denotes inside `krate::module` for `reach`.
    ///
    /// `path_start` admits extern-crate names, which only the first segment of
    /// a path may use. A cyclic lookup contributes nothing on its inner turn.
    pub(super) fn lookup(
        &self,
        krate: CrateId,
        module: &[String],
        name: &str,
        reach: Reach,
        path_start: bool,
    ) -> Result<Vec<Definition>, String> {
        let key = LookupKey {
            krate,
            module: module.to_vec(),
            name: name.to_owned(),
            reach,
            path_start,
        };
        if let Some(found) = self.memo.borrow().get(&key) {
            return Ok(found.clone());
        }
        if !self.active.borrow_mut().insert(key.clone()) {
            return Ok(Vec::new());
        }
        let found = self.lookup_uncached(krate, module, name, reach, path_start);
        self.active.borrow_mut().remove(&key);
        let mut found = found?;
        found.sort();
        found.dedup();
        self.memo.borrow_mut().insert(key, found.clone());
        Ok(found)
    }

    fn lookup_uncached(
        &self,
        krate: CrateId,
        module: &[String],
        name: &str,
        reach: Reach,
        path_start: bool,
    ) -> Result<Vec<Definition>, String> {
        let indexed = self.crates.crate_at(krate);
        let Some(node) = indexed.graph.modules.get(module) else {
            return Ok(Vec::new());
        };
        let items = node
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| !is_test_only(super::item_shape::attributes(item)))
            .collect::<Vec<_>>();

        let mut found = Vec::new();
        for (index, item) in &items {
            found.extend(self.explicit(krate, module, *index, item, name, reach)?);
        }
        if found.is_empty() {
            // A declared item wins: a macro that only adds `impl` blocks to it
            // (or fails to compile beside it) is not its definition.
            found.extend(self.generated(krate, module, name)?);
        }
        if !found.is_empty() {
            return Ok(found);
        }

        let mut outside_globs = Vec::new();
        for (_, item) in &items {
            let Item::Use(item_use) = item else { continue };
            if !admits(&item_use.vis, reach) {
                continue;
            }
            for binding in use_bindings(item_use) {
                if binding.bound == BoundName::Glob {
                    found.extend(self.through_glob(
                        krate,
                        module,
                        &binding,
                        name,
                        reach,
                        &mut outside_globs,
                    )?);
                }
            }
        }
        if module.is_empty() {
            found.extend(
                indexed
                    .exported_macros
                    .iter()
                    .filter(|(path, index)| {
                        macro_rules_name(&indexed.graph.modules[path].items[*index]).as_deref()
                            == Some(name)
                    })
                    .map(|(path, index)| Definition::Item {
                        krate,
                        module: path.clone(),
                        index: *index,
                    }),
            );
        }
        if found.is_empty() && path_start {
            found.extend(self.extern_root(krate, name)?);
        }
        if found.is_empty() {
            found.extend(outside_globs);
        }
        Ok(found)
    }

    /// What one explicit item or import contributes under `name`.
    fn explicit(
        &self,
        krate: CrateId,
        module: &[String],
        index: usize,
        item: &Item,
        name: &str,
        reach: Reach,
    ) -> Result<Vec<Definition>, String> {
        let mut found = Vec::new();
        match item {
            Item::Mod(item_mod) if item_mod.ident == name && admits(&item_mod.vis, reach) => {
                let mut path = module.to_vec();
                path.push(name.to_owned());
                if self
                    .crates
                    .crate_at(krate)
                    .graph
                    .modules
                    .contains_key(&path)
                {
                    found.push(module_definition(krate, &path));
                }
            }
            Item::Use(item_use) if admits(&item_use.vis, reach) => {
                for binding in use_bindings(item_use) {
                    if binding.bound == BoundName::Named(name.to_owned()) {
                        found.extend(self.resolve_path(krate, module, &binding.segments)?);
                    }
                }
            }
            Item::ExternCrate(extern_crate) if admits(&extern_crate.vis, reach) => {
                let bound = extern_crate
                    .rename
                    .as_ref()
                    .map_or(&extern_crate.ident, |(_, rename)| rename);
                if bound == name {
                    found.extend(self.extern_root(krate, &extern_crate.ident.to_string())?);
                }
            }
            Item::Macro(_) if reach != Reach::PublicOnly => {
                if macro_rules_name(item).as_deref() == Some(name) {
                    found.push(Definition::Item {
                        krate,
                        module: module.to_vec(),
                        index,
                    });
                }
            }
            _ => {
                if let Some((defined, vis)) = defined_name(item) {
                    if defined == name && admits(vis, reach) {
                        found.push(Definition::Item {
                            krate,
                            module: module.to_vec(),
                            index,
                        });
                    }
                }
            }
        }
        Ok(found)
    }

    fn through_glob(
        &self,
        krate: CrateId,
        module: &[String],
        binding: &UseBinding,
        name: &str,
        reach: Reach,
        outside_globs: &mut Vec<Definition>,
    ) -> Result<Vec<Definition>, String> {
        let mut found = Vec::new();
        for target in self.resolve_path(krate, module, &binding.segments)? {
            match target {
                Definition::Module {
                    krate: target_crate,
                    path,
                } => {
                    let inner_reach = if target_crate != krate {
                        Reach::PublicOnly
                    } else if module.starts_with(&path) {
                        reach
                    } else {
                        reach.max(Reach::CrateVisible)
                    };
                    found.extend(self.lookup(target_crate, &path, name, inner_reach, false)?);
                }
                Definition::Item {
                    krate: target_crate,
                    module: target_module,
                    index,
                } => found.extend(self.enum_variant(target_crate, &target_module, index, name)),
                Definition::OutsideWorkspace(path) => {
                    outside_globs.push(Definition::OutsideWorkspace(format!("{path}::{name}")));
                }
                Definition::Variant { .. } | Definition::Generated { .. } => {}
            }
        }
        Ok(found)
    }
}

/// Whether an item with `vis` is nameable under `reach`.
fn admits(vis: &Visibility, reach: Reach) -> bool {
    match reach {
        Reach::AnyVisibility => true,
        Reach::CrateVisible => !matches!(vis, Visibility::Inherited),
        Reach::PublicOnly => matches!(vis, Visibility::Public(_)),
    }
}
