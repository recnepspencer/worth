use std::collections::BTreeSet;
use syn::{
    visit::{self, Visit},
    Visibility,
};

pub(super) fn names(source: &str) -> Result<BTreeSet<String>, String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    RejectUninspectable::check(&file)?;
    let mut types = PublicTypes::default();
    types.visit_file(&file);
    let mut names = PublicNames {
        types: types.0.clone(),
        names: types.0,
        receiver: None,
    };
    names.visit_file(&file);
    Ok(names.names)
}
#[derive(Default)]
struct PublicTypes(BTreeSet<String>);
impl<'ast> Visit<'ast> for PublicTypes {
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.0.insert(item.ident.to_string());
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.0.insert(item.ident.to_string());
        }
        visit::visit_item_mod(self, item);
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.0.insert(item.ident.to_string());
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.0.insert(item.ident.to_string());
        }
    }
}
struct PublicNames {
    types: BTreeSet<String>,
    names: BTreeSet<String>,
    receiver: Option<String>,
}
impl<'ast> Visit<'ast> for PublicNames {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let previous = self.receiver.take();
        self.receiver = match item.self_ty.as_ref() {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            _ => None,
        };
        visit::visit_item_impl(self, item);
        self.receiver = previous;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if !matches!(item.vis, Visibility::Public(_)) {
            return;
        }
        let name = item.sig.ident.to_string();
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
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.names.insert(item.sig.ident.to_string());
        }
    }
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.names.insert(item.ident.to_string());
        }
    }
    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.names.insert(item.ident.to_string());
        }
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if matches!(item.vis, Visibility::Public(_)) {
            self.names.insert(item.ident.to_string());
        }
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if matches!(item.vis, Visibility::Public(_)) {
            reexports(&item.tree, &mut self.names);
        }
    }
}
fn reexports(tree: &syn::UseTree, names: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(path) => reexports(&path.tree, names),
        syn::UseTree::Name(name) => {
            names.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            names.insert(rename.rename.to_string());
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                reexports(tree, names);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

// Fail closed when a future control uses an unexpanded public surface.
// External module children are covered recursively by control_seal::validate.
#[derive(Default)]
struct RejectUninspectable(Option<String>);
impl RejectUninspectable {
    fn check(file: &syn::File) -> Result<(), String> {
        let mut check = Self::default();
        check.visit_file(file);
        check.0.map_or(Ok(()), Err)
    }
}
impl<'ast> Visit<'ast> for RejectUninspectable {
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        fn has_public(tokens: proc_macro2::TokenStream) -> bool {
            tokens.into_iter().any(|token| match token {
                proc_macro2::TokenTree::Ident(name) => name == "pub",
                proc_macro2::TokenTree::Group(group) => has_public(group.stream()),
                _ => false,
            })
        }
        if has_public(item.mac.tokens.clone()) {
            self.0 = Some("public control items inside macros must be declared explicitly".into());
        }
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if matches!(item.vis, Visibility::Public(_))
            && quote::quote!(#item).to_string().contains('*')
        {
            self.0 = Some("glob reexports cannot define a sealed control surface".into());
        }
    }
}
