//! What an isolated owner declares: its types, visible values and the
//! visible methods it adds to types declared elsewhere. Test-only items, and
//! the files of a `#[cfg(test)]` module, bind nothing.

use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;

use super::test_sources::is_test_only;

/// What an isolated owner declares.
#[derive(Default)]
pub(super) struct OwnedItems {
    /// Type names, refused wherever they appear.
    pub(super) types: BTreeSet<String>,
    /// Crate-visible function, constant, static and exported macro names,
    /// refused when a qualified path or an import reaches them.
    pub(super) values: BTreeSet<String>,
    /// Values visible only inside a module, keyed by name to each module that
    /// bounds one; refused only from guarded code inside that module.
    pub(super) scoped_values: BTreeMap<String, Vec<Vec<String>>>,
    /// Crate-visible methods the owner adds to types it does not own,
    /// refused when a method call or a qualified path reaches them.
    pub(super) methods: BTreeSet<String>,
    /// Such methods visible only inside a module, keyed like `scoped_values`.
    pub(super) scoped_methods: BTreeMap<String, Vec<Vec<String>>>,
}

/// A foreign-type method the owner declares: its receiver type, its name and
/// the module its visibility bounds, `None` for the whole crate.
pub(super) type DeclaredMethod = (String, String, Option<Vec<String>>);

impl OwnedItems {
    /// Whether code in `module` can name the owned value `name`.
    pub(super) fn reaches_value(&self, name: &str, module: &[String]) -> bool {
        self.values.contains(name) || within(&self.scoped_values, name, module)
    }

    /// Whether code in `module` can call the owned foreign-type method `name`.
    pub(super) fn reaches_method(&self, name: &str, module: &[String]) -> bool {
        self.methods.contains(name) || within(&self.scoped_methods, name, module)
    }

    /// Keeps the methods whose receiver type the owner does not declare at
    /// any visibility; a method on the owner's own type is unreachable without
    /// first reaching that type.
    pub(super) fn add_foreign_type_methods(
        &mut self,
        methods: Vec<DeclaredMethod>,
        declared: &BTreeSet<String>,
    ) {
        for (self_type, method, scope) in methods {
            if declared.contains(&self_type) {
                continue;
            }
            match scope {
                Some(scope) => self.scoped_methods.entry(method).or_default().push(scope),
                None => {
                    self.methods.insert(method);
                }
            }
        }
    }
}

fn within(scoped: &BTreeMap<String, Vec<Vec<String>>>, name: &str, module: &[String]) -> bool {
    scoped
        .get(name)
        .is_some_and(|scopes| scopes.iter().any(|scope| module.starts_with(scope)))
}

pub(super) struct OwnedItemCollector<'a> {
    pub(super) owned: &'a mut OwnedItems,
    /// Every type name the owner declares, whatever its visibility.
    pub(super) declared: &'a mut BTreeSet<String>,
    /// Visible inherent methods with the receiver type they extend.
    pub(super) methods: &'a mut Vec<DeclaredMethod>,
    /// The crate-relative module being visited.
    pub(super) module: Vec<String>,
}

impl OwnedItemCollector<'_> {
    fn value(&mut self, visibility: &syn::Visibility, identifier: &proc_macro2::Ident) {
        match self.visible_in(visibility) {
            None => {}
            Some(Some(scope)) => self
                .owned
                .scoped_values
                .entry(identifier.to_string())
                .or_default()
                .push(scope),
            Some(None) => {
                self.owned.values.insert(identifier.to_string());
            }
        }
    }

    /// `None` for a private item, else the module its visibility bounds, or
    /// `Some(None)` for one the whole crate can name.
    fn visible_in(&self, visibility: &syn::Visibility) -> Option<Option<Vec<String>>> {
        match visibility {
            syn::Visibility::Inherited => None,
            syn::Visibility::Public(_) => Some(None),
            syn::Visibility::Restricted(restricted) => Some(self.scope(&restricted.path)),
        }
    }

    /// The module a `pub(...)` restriction bounds, or `None` for the crate.
    fn scope(&self, path: &syn::Path) -> Option<Vec<String>> {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let scope = match segments.first().map(String::as_str) {
            Some("crate") => segments[1..].to_vec(),
            Some("self") => [&self.module[..], &segments[1..]].concat(),
            Some("super") => {
                let depth = segments.iter().take_while(|s| *s == "super").count();
                let kept = self.module.len().checked_sub(depth)?;
                [&self.module[..kept], &segments[depth..]].concat()
            }
            _ => return None,
        };
        (!scope.is_empty()).then_some(scope)
    }

    fn owned_type(&mut self, visibility: &syn::Visibility, identifier: &proc_macro2::Ident) {
        self.declared.insert(identifier.to_string());
        if !matches!(visibility, syn::Visibility::Inherited) {
            self.owned.types.insert(identifier.to_string());
        }
    }
}

impl Visit<'_> for OwnedItemCollector<'_> {
    fn visit_item(&mut self, item: &syn::Item) {
        let attributes = match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::ExternCrate(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::ForeignMod(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Macro(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::TraitAlias(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            _ => return syn::visit::visit_item(self, item),
        };
        if !is_test_only(attributes) {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &syn::ItemMod) {
        let inline = item.content.is_some();
        if inline {
            self.module.push(item.ident.to_string());
        }
        syn::visit::visit_item_mod(self, item);
        if inline {
            self.module.pop();
        }
    }

    fn visit_item_struct(&mut self, item: &syn::ItemStruct) {
        self.owned_type(&item.vis, &item.ident);
    }

    fn visit_item_enum(&mut self, item: &syn::ItemEnum) {
        self.owned_type(&item.vis, &item.ident);
    }

    fn visit_item_trait(&mut self, item: &syn::ItemTrait) {
        self.owned_type(&item.vis, &item.ident);
    }

    fn visit_item_type(&mut self, item: &syn::ItemType) {
        self.owned_type(&item.vis, &item.ident);
    }

    fn visit_item_union(&mut self, item: &syn::ItemUnion) {
        self.owned_type(&item.vis, &item.ident);
    }

    fn visit_item_impl(&mut self, item: &syn::ItemImpl) {
        let syn::Type::Path(self_type) = item.self_ty.as_ref() else {
            return;
        };
        let (None, Some(receiver)) = (&item.trait_, self_type.path.segments.last()) else {
            return;
        };
        for member in &item.items {
            if let syn::ImplItem::Fn(method) = member {
                if is_test_only(&method.attrs) {
                    continue;
                }
                if let Some(scope) = self.visible_in(&method.vis) {
                    self.methods.push((
                        receiver.ident.to_string(),
                        method.sig.ident.to_string(),
                        scope,
                    ));
                }
            }
        }
    }

    fn visit_item_fn(&mut self, item: &syn::ItemFn) {
        self.value(&item.vis, &item.sig.ident);
    }

    fn visit_item_const(&mut self, item: &syn::ItemConst) {
        self.value(&item.vis, &item.ident);
    }

    fn visit_item_static(&mut self, item: &syn::ItemStatic) {
        self.value(&item.vis, &item.ident);
    }

    fn visit_item_macro(&mut self, item: &syn::ItemMacro) {
        let exported = item
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("macro_export"));
        if let (true, Some(identifier)) = (exported, &item.ident) {
            self.owned.values.insert(identifier.to_string());
        }
    }
}
