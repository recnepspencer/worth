//! Only Execution may access raw allocation variants or create an alias lane.
use super::super::production_scope::{
    visit_field_in_scope, visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope,
    visit_variant_in_scope, ScopedVisitor,
};
use super::raw_kind_types::{Aliases, RAW};
use proc_macro2::{Span, TokenStream};
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ImplItem, Item, TraitItem, UseTree};

pub(super) fn findings(
    items: &[Item],
    aliases: &Aliases,
    source: &str,
) -> BTreeSet<(String, usize)> {
    let mut visitor = RawAccess {
        scope: aliases.namespace(source),
        found: BTreeSet::new(),
        aliases,
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found
}

struct RawAccess<'a> {
    scope: Vec<String>,
    found: BTreeSet<(String, usize)>,
    aliases: &'a Aliases,
}

impl ScopedVisitor for RawAccess<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl RawAccess<'_> {
    fn refuse(&mut self, span: Span) {
        self.found
            .insert((self.scope.join("::"), span.start().line));
    }

    fn qualified_access(&mut self, qself: Option<&syn::QSelf>, path: &syn::Path) -> bool {
        if let Some(qself) = qself {
            if path.segments.len() > qself.position
                && self.aliases.mentions_raw(&qself.ty, &self.scope)
            {
                self.refuse(qself.ty.span());
                return true;
            }
        }
        false
    }

    fn path_access(&mut self, path: &syn::Path) {
        let mut prefix = syn::Path {
            leading_colon: path.leading_colon,
            segments: syn::punctuated::Punctuated::new(),
        };
        for segment in path
            .segments
            .iter()
            .take(path.segments.len().saturating_sub(1))
        {
            prefix.segments.push(segment.clone());
            let ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: prefix.clone(),
            });
            if self.aliases.mentions_raw(&ty, &self.scope) {
                self.refuse(segment.ident.span());
            }
        }
    }

    fn macro_access(&mut self, tokens: TokenStream) {
        // Parsed items own their descendant groups. Recurse through tokens only
        // when this fragment is not an item list, so each body is scanned once.
        if let Ok(file) = syn::parse2::<syn::File>(tokens.clone()) {
            for item in &file.items {
                self.visit_item(item);
            }
            return;
        }
        // Expression arguments also own their descendants once parsed. This
        // covers literal aliases in matches!, including nested macro calls.
        if let Ok(expressions) = syn::parse::Parser::parse2(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            tokens.clone(),
        ) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
            return;
        }
        let tokens: Vec<_> = tokens.into_iter().collect();
        // A DSL fragment owns each token group once; path candidates are
        // disjoint, rather than reparsing every possible start/end pair.
        super::macro_paths::scan(&tokens, |fragment| match fragment {
            super::macro_paths::Fragment::RawMember(span) => self.refuse(span),
            super::macro_paths::Fragment::Path(path) => {
                if !self.qualified_access(path.qself.as_ref(), &path.path) {
                    self.path_access(&path.path);
                }
            }
            super::macro_paths::Fragment::Group(tokens) => self.macro_access(tokens),
        });
    }
}

impl<'ast> Visit<'ast> for RawAccess<'_> {
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
        self.path_access(path);
        visit::visit_path(self, path);
    }
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if self.qualified_access(path.qself.as_ref(), &path.path) {
            return;
        }
        visit::visit_expr_path(self, path);
    }
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if self.qualified_access(path.qself.as_ref(), &path.path) {
            return;
        }
        visit::visit_type_path(self, path);
    }
    fn visit_pat_tuple_struct(&mut self, pat: &'ast syn::PatTupleStruct) {
        if self.qualified_access(pat.qself.as_ref(), &pat.path) {
            for elem in &pat.elems {
                self.visit_pat(elem);
            }
        } else {
            visit::visit_pat_tuple_struct(self, pat);
        }
    }
    fn visit_pat_struct(&mut self, pat: &'ast syn::PatStruct) {
        if self.qualified_access(pat.qself.as_ref(), &pat.path) {
            for field in &pat.fields {
                self.visit_field_pat(field);
            }
        } else {
            visit::visit_pat_struct(self, pat);
        }
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
        if self.aliases.raw_type(&item.ty, &self.scope) {
            self.refuse(item.ident.span());
        }
        visit::visit_item_type(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        if self.aliases.raw_type(&item.ty, &self.scope) {
            self.refuse(item.ident.span());
        }
        visit::visit_impl_item_type(self, item);
    }
    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        if item
            .default
            .as_ref()
            .is_some_and(|(_, ty)| self.aliases.raw_type(ty, &self.scope))
        {
            self.refuse(item.ident.span());
        }
        visit::visit_trait_item_type(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if item.trait_.is_some() && self.aliases.raw_type(&item.self_ty, &self.scope) {
            self.refuse(item.self_ty.span());
        }
        visit::visit_item_impl(self, item);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.macro_access(mac.tokens.clone());
    }
}
