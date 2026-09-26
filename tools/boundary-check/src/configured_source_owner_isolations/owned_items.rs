//! What an isolated owner declares: its types, visible values and the
//! visible methods it adds to types declared elsewhere. Test-only items, and
//! the files of a `#[cfg(test)]` module, bind nothing.

use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;

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
    /// Visible methods the owner adds to types it does not own, refused when
    /// a method call or a qualified path reaches them.
    pub(super) methods: BTreeSet<String>,
}

impl OwnedItems {
    /// Whether code in `module` can name the owned value `name`.
    pub(super) fn reaches_value(&self, name: &str, module: &[String]) -> bool {
        self.values.contains(name)
            || self
                .scoped_values
                .get(name)
                .is_some_and(|scopes| scopes.iter().any(|scope| module.starts_with(scope)))
    }

    /// Keeps the methods whose receiver type the owner does not declare at
    /// any visibility; a method on the owner's own type is unreachable without
    /// first reaching that type.
    pub(super) fn add_foreign_type_methods(
        &mut self,
        methods: Vec<(String, String)>,
        declared: &BTreeSet<String>,
    ) {
        for (self_type, method) in methods {
            if !declared.contains(&self_type) {
                self.methods.insert(method);
            }
        }
    }
}

/// The module path of every file an out-of-line `#[cfg(test)]` module
/// declares, written as `dir/name` without `.rs`. Guarded production code
/// cannot reach such a file or any file below it. A file's name alone never
/// makes it a test: only the declaration does.
pub(super) fn test_module_roots(sources: &[(String, syn::File)]) -> Vec<String> {
    let mut roots = Vec::new();
    for (path, file) in sources {
        let path = path.replace('\\', "/");
        let (directory, name) = path.rsplit_once('/').unwrap_or(("", path.as_str()));
        let stem = name.trim_end_matches(".rs");
        let children = if matches!(stem, "mod" | "lib" | "main") {
            directory.to_owned()
        } else {
            format!("{directory}/{stem}")
        };
        for item in &file.items {
            let syn::Item::Mod(module) = item else {
                continue;
            };
            if module.content.is_some() || !is_test_only(&module.attrs) {
                continue;
            }
            let declared = match path_attribute(&module.attrs) {
                Some(relative) => format!("{directory}/{relative}"),
                None => format!("{children}/{}", module.ident),
            };
            roots.push(module_stem(&declared).to_owned());
        }
    }
    roots
}

/// Whether an owner source belongs to a `#[cfg(test)]` module.
pub(super) fn is_test_source(path: &str, roots: &[String]) -> bool {
    let path = path.replace('\\', "/");
    let stem = module_stem(&path);
    roots.iter().any(|root| {
        stem == root
            || stem
                .strip_prefix(root.as_str())
                .is_some_and(|below| below.starts_with('/'))
    })
}

/// `dir/name.rs` and `dir/name/mod.rs` both name the module `dir/name`.
fn module_stem(path: &str) -> &str {
    let stem = path.trim_end_matches(".rs");
    stem.strip_suffix("/mod").unwrap_or(stem)
}

fn path_attribute(attributes: &[syn::Attribute]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                path,
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }),
                ..
            }) if path.is_ident("path") => Some(value.value()),
            _ => None,
        })
}

fn is_test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Ident>()
                .is_ok_and(|predicate| predicate == "test")
    })
}

pub(super) struct OwnedItemCollector<'a> {
    pub(super) owned: &'a mut OwnedItems,
    /// Every type name the owner declares, whatever its visibility.
    pub(super) declared: &'a mut BTreeSet<String>,
    /// Visible inherent methods, keyed by the receiver type they extend.
    pub(super) methods: &'a mut Vec<(String, String)>,
    /// The crate-relative module being visited.
    pub(super) module: Vec<String>,
}

impl OwnedItemCollector<'_> {
    fn value(&mut self, visibility: &syn::Visibility, identifier: &proc_macro2::Ident) {
        match visibility {
            syn::Visibility::Inherited => {}
            syn::Visibility::Public(_) => {
                self.owned.values.insert(identifier.to_string());
            }
            syn::Visibility::Restricted(restricted) => match self.scope(&restricted.path) {
                Some(scope) => self
                    .owned
                    .scoped_values
                    .entry(identifier.to_string())
                    .or_default()
                    .push(scope),
                None => {
                    self.owned.values.insert(identifier.to_string());
                }
            },
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
                if !matches!(method.vis, syn::Visibility::Inherited) && !is_test_only(&method.attrs)
                {
                    self.methods
                        .push((receiver.ident.to_string(), method.sig.ident.to_string()));
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
