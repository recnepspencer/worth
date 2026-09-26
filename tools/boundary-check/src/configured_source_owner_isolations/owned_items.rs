//! What an isolated owner declares: its types, visible values and the
//! visible methods it adds to types declared elsewhere. Test-only items and
//! test sources bind nothing.

use std::collections::BTreeSet;

use syn::visit::Visit;

/// What an isolated owner declares.
#[derive(Default)]
pub(super) struct OwnedItems {
    /// Type names, refused wherever they appear.
    pub(super) types: BTreeSet<String>,
    /// Visible function, constant, static and exported macro names, refused
    /// when a qualified path or an import reaches them.
    pub(super) values: BTreeSet<String>,
    /// Visible methods the owner adds to types it does not own, refused when
    /// a method call or a qualified path reaches them.
    pub(super) methods: BTreeSet<String>,
}

impl OwnedItems {
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

/// Whether an owner source is a test module, which guarded production code
/// cannot reach.
pub(super) fn is_test_source(path: &str) -> bool {
    path.split(['/', '\\'])
        .any(|component| component == "tests" || component == "tests.rs")
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
}

impl OwnedItemCollector<'_> {
    fn value(&mut self, visibility: &syn::Visibility, identifier: &proc_macro2::Ident) {
        if !matches!(visibility, syn::Visibility::Inherited) {
            self.owned.values.insert(identifier.to_string());
        }
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
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            _ => return syn::visit::visit_item(self, item),
        };
        if !is_test_only(attributes) {
            syn::visit::visit_item(self, item);
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
