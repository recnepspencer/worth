//! The private installation omission door has file-exact ownership and callers.
use super::{failure, Doors};
use crate::diagnostics::Diagnostic;
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

pub(super) const OWNER: &str =
    "crates/worth-cad-entry/src/contribution/installation_omission_for_test.rs";
const MOUNT: &str = "crates/worth-cad-entry/src/contribution.rs";
const CALLER: &str = "crates/worth-cad-entry/tests/program_declaration/installation.rs";

pub(super) fn contains(path: &str) -> bool {
    path.starts_with("crates/worth-cad-entry/")
}

pub(super) fn may_declare(path: &str) -> bool {
    !contains(path) || path == OWNER || path == MOUNT
}

pub(super) fn names(doors: &Doors) -> BTreeSet<String> {
    doors
        .iter()
        .filter(|(_, owners)| owners.contains(OWNER))
        .map(|(name, _)| name.clone())
        .collect()
}

pub(super) fn alias_denials(path: &str, source: &str, names: &BTreeSet<String>) -> Vec<Diagnostic> {
    if names.is_empty() || (path != OWNER && !names.iter().any(|name| source.contains(name))) {
        return Vec::new();
    }
    // Noncallers already receive the ordinary reach refusal. Item-level checks
    // seal the admitted files using that same policy, not a second caller list.
    if !super::permitted(path, &BTreeSet::from([OWNER.to_owned()])) {
        return Vec::new();
    }
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => {
            return vec![failure(
                path,
                format!("cannot inspect CAD control aliases: {error}"),
            )]
        }
    };
    let mut aliases = Aliases {
        path,
        names,
        found: BTreeSet::new(),
        unmarked_exports: BTreeSet::new(),
        control_receiver: false,
        item_depth: 0,
    };
    aliases.visit_file(&file);
    let mut diagnostics: Vec<_> = aliases
        .found
        .into_iter()
        .map(|name| {
            failure(
                path,
                format!("exact CAD control `{name}` may not be aliased"),
            )
        })
        .collect();
    diagnostics.extend(aliases.unmarked_exports.into_iter().map(|name| {
        failure(
            path,
            format!(
                "CAD control implementation export `{name}` must use the ForTest naming convention"
            ),
        )
    }));
    diagnostics
}

struct Aliases<'a> {
    path: &'a str,
    names: &'a BTreeSet<String>,
    found: BTreeSet<String>,
    unmarked_exports: BTreeSet<String>,
    control_receiver: bool,
    // Structural nesting identifies the two file-level registration items;
    // this is not a recursion/traversal limit.
    item_depth: usize,
}
impl Aliases<'_> {
    fn record(&mut self, tokens: proc_macro2::TokenStream) {
        super::collect(tokens, self.names, &mut self.found);
    }
    fn private_implementation(&self, visibility: &syn::Visibility) -> bool {
        matches!(visibility, syn::Visibility::Inherited) && matches!(self.path, OWNER | CALLER)
    }
    fn owner_export(&mut self, visibility: &syn::Visibility, name: &syn::Ident) {
        if self.path == OWNER
            && !matches!(visibility, syn::Visibility::Inherited)
            && !super::public_items::test_name(&name.to_string())
        {
            self.unmarked_exports.insert(name.to_string());
        }
    }
}
impl<'ast> Visit<'ast> for Aliases<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        self.item_depth += 1;
        visit::visit_item(self, item);
        self.item_depth -= 1;
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.owner_export(&item.vis, &item.sig.ident);
        let owner_door =
            self.path == OWNER && super::public_items::test_name(&item.sig.ident.to_string());
        let hook =
            self.item_depth == 1 && registration_hook(self.path, &item.sig.ident.to_string());
        if !self.private_implementation(&item.vis) && !owner_door && !hook {
            self.record(item.sig.to_token_stream());
            self.record(item.block.to_token_stream());
        }
        visit::visit_item_fn(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let previous = self.control_receiver;
        self.control_receiver = self.path == OWNER
            && matches!(item.self_ty.as_ref(), syn::Type::Path(path)
            if path.path.segments.last().is_some_and(|s| self.names.contains(&s.ident.to_string())));
        visit::visit_item_impl(self, item);
        self.control_receiver = previous;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if !self.control_receiver {
            self.owner_export(&item.vis, &item.sig.ident);
        }
        let owner_door =
            self.path == OWNER && super::public_items::test_name(&item.sig.ident.to_string());
        if !self.control_receiver && !self.private_implementation(&item.vis) && !owner_door {
            self.record(item.sig.to_token_stream());
            self.record(item.block.to_token_stream());
        }
        visit::visit_impl_item_fn(self, item);
    }
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if !matches!(self.path, OWNER | CALLER) {
            self.record(item.to_token_stream());
        }
        visit::visit_trait_item_fn(self, item);
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.owner_export(&item.vis, &item.ident);
        visit::visit_item_struct(self, item);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.owner_export(&item.vis, &item.ident);
        visit::visit_item_enum(self, item);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.owner_export(&item.vis, &item.ident);
        visit::visit_item_trait(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.owner_export(&item.vis, &item.ident);
        visit::visit_item_mod(self, item);
    }
    fn visit_use_rename(&mut self, rename: &'ast syn::UseRename) {
        self.record(rename.ident.to_token_stream());
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.owner_export(&item.vis, &item.ident);
        self.record(item.ty.to_token_stream());
    }
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        self.owner_export(&item.vis, &item.ident);
        if !self.private_implementation(&item.vis) {
            self.record(item.ty.to_token_stream());
            self.record(item.expr.to_token_stream());
        }
        visit::visit_item_const(self, item);
    }
    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        self.owner_export(&item.vis, &item.ident);
        if !self.private_implementation(&item.vis) {
            self.record(item.ty.to_token_stream());
            self.record(item.expr.to_token_stream());
        }
        visit::visit_item_static(self, item);
    }
}

fn registration_hook(path: &str, name: &str) -> bool {
    matches!(
        (path, name),
        (
            "crates/worth-cad-entry/src/families/chamfer/realization/mod.rs",
            "configure"
        ) | (
            "crates/worth-cad-entry/src/schema/chamfer_correspondence.rs",
            "declare_chamfer_correspondence"
        )
    )
}
