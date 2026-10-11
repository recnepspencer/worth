//! Only Execution may access raw allocation variants or create an alias lane.
use super::super::production_scope::{
    visit_field_in_scope, visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope,
    visit_variant_in_scope, ScopedVisitor,
};
use proc_macro2::{Span, TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ImplItem, Item, TraitItem, Type, UseTree};

const RAW: &str = "ExecutionAllocationDenialKind";

pub(super) fn findings(items: &[Item]) -> BTreeSet<(String, usize)> {
    let mut visitor = RawAccess {
        scope: Vec::new(),
        found: BTreeSet::new(),
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found
}

struct RawAccess {
    scope: Vec<String>,
    found: BTreeSet<(String, usize)>,
}

impl ScopedVisitor for RawAccess {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl RawAccess {
    fn refuse(&mut self, span: Span) {
        self.found
            .insert((self.scope.join("::"), span.start().line));
    }

    fn qualified_access(&mut self, qself: Option<&syn::QSelf>, path: &syn::Path) {
        if let Some(qself) = qself {
            if path.segments.len() > qself.position && raw_self_type(&qself.ty) {
                self.refuse(qself.ty.span());
            }
        }
    }

    fn macro_access(&mut self, tokens: TokenStream) {
        // Item-generating macros are governed too; parse complete item fragments,
        // then scan every group for the forbidden path spelling.
        if let Ok(file) = syn::parse2::<syn::File>(tokens.clone()) {
            for item in &file.items {
                self.visit_item(item);
            }
        }
        let tokens: Vec<_> = tokens.into_iter().collect();
        for window in tokens.windows(3) {
            if let [TokenTree::Ident(name), TokenTree::Punct(first), TokenTree::Punct(second)] =
                window
            {
                if name == RAW && first.as_char() == ':' && second.as_char() == ':' {
                    self.refuse(name.span());
                }
            }
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.macro_access(group.stream());
            }
        }
    }
}

impl<'ast> Visit<'ast> for RawAccess {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item);
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        visit_field_in_scope(self, field);
    }
    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        visit_variant_in_scope(self, variant);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in path
            .segments
            .iter()
            .take(path.segments.len().saturating_sub(1))
        {
            if segment.ident == RAW {
                self.refuse(segment.ident.span());
            }
        }
        visit::visit_path(self, path);
    }
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        self.qualified_access(path.qself.as_ref(), &path.path);
        visit::visit_expr_path(self, path);
    }
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        self.qualified_access(path.qself.as_ref(), &path.path);
        visit::visit_type_path(self, path);
    }
    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        match tree {
            UseTree::Path(path) if path.ident == RAW => self.refuse(path.ident.span()),
            // A renamed import creates the same alternate spelling as a type alias.
            UseTree::Rename(rename) if rename.ident == RAW && rename.rename != RAW => {
                self.refuse(rename.ident.span());
            }
            _ => {}
        }
        visit::visit_use_tree(self, tree);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if raw_self_type(&item.ty) {
            self.refuse(item.ident.span());
        }
        visit::visit_item_type(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        if raw_self_type(&item.ty) {
            self.refuse(item.ident.span());
        }
        visit::visit_impl_item_type(self, item);
    }
    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        if item
            .default
            .as_ref()
            .is_some_and(|(_, ty)| raw_self_type(ty))
        {
            self.refuse(item.ident.span());
        }
        visit::visit_trait_item_type(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if item.trait_.is_some() && raw_self_type(&item.self_ty) {
            self.refuse(item.self_ty.span());
        }
        visit::visit_item_impl(self, item);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.macro_access(mac.tokens.clone());
    }
}

fn raw_self_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == RAW),
        Type::Reference(reference) => raw_self_type(&reference.elem),
        Type::Paren(paren) => raw_self_type(&paren.elem),
        Type::Group(group) => raw_self_type(&group.elem),
        _ => false,
    }
}
