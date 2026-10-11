use std::collections::BTreeSet;
use syn::{
    visit::{self, Visit},
    Visibility,
};

// Law 1 uses the same spelling for raw and ordinary references.
fn spelling(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}

pub(super) fn test_names(source: &str) -> Result<BTreeSet<String>, String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut types = PublicTypes::default();
    types.visit_file(&file);
    let mut names = PublicNames {
        types: types.0.clone(),
        names: types.0,
        receiver: None,
    };
    names.visit_file(&file);
    Ok(names
        .names
        .into_iter()
        .filter(|name| test_name(name))
        .collect())
}

pub(super) fn test_name(name: &str) -> bool {
    name.ends_with("_for_test") || name.ends_with("ForTest")
}

fn exported(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
        || matches!(vis, Visibility::Restricted(restricted) if restricted.path.is_ident("crate"))
}
#[derive(Default)]
struct PublicTypes(BTreeSet<String>);
impl<'ast> Visit<'ast> for PublicTypes {
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    fn visit_impl_item_fn(&mut self, _: &'ast syn::ImplItemFn) {}
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.0.insert(spelling(&item.ident));
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.0.insert(spelling(&item.ident));
        }
        visit::visit_item_mod(self, item);
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.0.insert(spelling(&item.ident));
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.0.insert(spelling(&item.ident));
        }
    }
}
struct PublicNames {
    types: BTreeSet<String>,
    names: BTreeSet<String>,
    receiver: Option<String>,
}
impl<'ast> Visit<'ast> for PublicNames {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if !super::source_regions::test_only(&item.attrs) {
            visit::visit_item_struct(self, item);
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if !super::source_regions::test_only(&item.attrs) {
            visit::visit_item_enum(self, item);
        }
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        if exported(&field.vis) && !super::source_regions::test_only(&field.attrs) {
            if let Some(name) = &field.ident {
                self.names.insert(spelling(name));
            }
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !super::source_regions::test_only(&item.attrs) {
            visit::visit_item_mod(self, item);
        }
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        super::macro_items::names(item.mac.tokens.clone(), &mut self.names);
        if item.attrs.iter().any(|attr| {
            attr.path().segments.len() == 1
                && spelling(&attr.path().segments[0].ident) == "macro_export"
        }) {
            if let Some(name) = &item.ident {
                self.names.insert(spelling(name));
            }
        }
    }
    fn visit_impl_item_macro(&mut self, item: &'ast syn::ImplItemMacro) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        super::macro_items::names(item.mac.tokens.clone(), &mut self.names);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            // Law 1, reach: trait members are public without a `pub` token.
            for member in &item.items {
                match member {
                    syn::TraitItem::Fn(member) => {
                        self.names.insert(spelling(&member.sig.ident));
                    }
                    syn::TraitItem::Const(member) => {
                        self.names.insert(spelling(&member.ident));
                    }
                    syn::TraitItem::Type(member) => {
                        self.names.insert(spelling(&member.ident));
                    }
                    _ => {}
                }
            }
        }
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        let previous = self.receiver.take();
        self.receiver = match item.self_ty.as_ref() {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| spelling(&segment.ident)),
            _ => None,
        };
        visit::visit_item_impl(self, item);
        self.receiver = previous;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if !exported(&item.vis) {
            return;
        }
        let name = spelling(&item.sig.ident);
        // Law 1, reach: lifecycle members of a sealed control type cannot be
        // reached without a sealed acquisition door or a reference to that type.
        // Do not globally forbid ordinary APIs also named `release` or `new`.
        let sealed_receiver = self
            .receiver
            .as_ref()
            .is_some_and(|receiver| self.types.contains(receiver));
        let lifecycle = matches!(
            name.as_str(),
            "release"
                | "new"
                | "wait_until_reached"
                | "wait_until_joined"
                | "wait_until_reader_contended"
        );
        if !(sealed_receiver && lifecycle) {
            self.names.insert(name);
        }
    }
    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_foreign_item_fn(&mut self, item: &'ast syn::ForeignItemFn) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.sig.ident));
        }
    }
    fn visit_foreign_item_static(&mut self, item: &'ast syn::ForeignItemStatic) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.sig.ident));
        }
    }
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            self.names.insert(spelling(&item.ident));
        }
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if super::source_regions::test_only(&item.attrs) {
            return;
        }
        if exported(&item.vis) {
            reexports(&item.tree, &mut self.names);
        }
    }
}
fn reexports(tree: &syn::UseTree, names: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(path) => reexports(&path.tree, names),
        syn::UseTree::Name(name) => {
            names.insert(spelling(&name.ident));
        }
        syn::UseTree::Rename(rename) => {
            names.insert(spelling(&rename.rename));
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                reexports(tree, names);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}
