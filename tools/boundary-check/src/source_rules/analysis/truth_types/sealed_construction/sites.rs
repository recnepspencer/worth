//! Constructions of sealed types outside their owners, and the suppressions
//! that admit a mint Clippy enforces.

use super::super::super::production_cfg::compiled_out_of_production;
use super::super::super::production_scope::{
    self_type_name, visit_field_in_scope, visit_impl_item_in_scope, visit_item_in_scope,
    visit_trait_item_in_scope, visit_variant_in_scope, ProductionModule, ScopedVisitor,
};
use super::owned_types;
use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ImplItem, Item, Type, UseTree};

/// Every site in `module` that constructs a sealed type it does not own, or
/// suppresses Clippy's disallowed methods.
pub(super) fn read(
    owners: &BTreeMap<&str, &[String]>,
    constructors: &BTreeMap<String, BTreeSet<String>>,
    module: &ProductionModule<'_>,
) -> Vec<Site> {
    let mut reader = ConstructionReader {
        owners,
        owned: owned_types(owners, module),
        constructors,
        scope: Vec::new(),
        sites: Vec::new(),
    };
    module
        .attributes
        .iter()
        .for_each(|attribute| reader.visit_attribute(attribute));
    module.items.iter().for_each(|item| reader.visit_item(item));
    reader.sites
}

pub(super) enum SiteKind {
    Call,
    Suppression { expect: bool },
    Refused(String),
}

pub(super) struct Site {
    pub(super) item: String,
    pub(super) call: String,
    pub(super) line: usize,
    pub(super) kind: SiteKind,
}

struct ConstructionReader<'a> {
    owners: &'a BTreeMap<&'a str, &'a [String]>,
    owned: BTreeSet<&'a str>,
    constructors: &'a BTreeMap<String, BTreeSet<String>>,
    scope: Vec<String>,
    sites: Vec<Site>,
}

impl ScopedVisitor for ConstructionReader<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl ConstructionReader<'_> {
    /// A sealed type this module does not own.
    fn foreign(&self, name: &str) -> bool {
        self.owners.contains_key(name) && !self.owned.contains(name)
    }

    fn site(&mut self, line: usize, call: String, kind: SiteKind) {
        self.sites.push(Site {
            item: self.scope.join("::"),
            call,
            line,
            kind,
        });
    }

    fn refuse(&mut self, line: usize, message: String) {
        self.site(line, String::new(), SiteKind::Refused(message));
    }

    /// A path in expression position: a constructor call, or a tuple
    /// constructor named as a value.
    fn read_path(&mut self, segments: &[String], line: usize, literal: bool) {
        if let [.., last] = segments {
            if self.foreign(last) {
                let form = if literal {
                    "literal"
                } else {
                    "tuple constructor"
                };
                self.refuse(
                    line,
                    format!("builds sealed `{last}` by its {form} outside its owner"),
                );
                return;
            }
        }
        if let [.., owner, name] = segments {
            let constructs = self
                .constructors
                .get(owner)
                .is_some_and(|names| names.contains(name));
            if constructs && self.foreign(owner) {
                self.site(line, format!("{owner}::{name}"), SiteKind::Call);
            }
        }
    }

    fn read_use(&mut self, tree: &UseTree, line: usize) {
        match tree {
            UseTree::Path(path) if self.foreign(&path.ident.to_string()) => self.refuse(
                line,
                format!("imports through sealed `{}` outside its owner; name its constructors by their type", path.ident),
            ),
            UseTree::Path(path) => self.read_use(&path.tree, line),
            UseTree::Rename(rename) if self.foreign(&rename.ident.to_string()) => self.refuse(
                line,
                format!("renames sealed `{}` to `{}` outside its owner", rename.ident, rename.rename),
            ),
            UseTree::Group(group) => group.items.iter().for_each(|tree| self.read_use(tree, line)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    fn read_tokens(&mut self, tokens: TokenStream) {
        let tokens: Vec<TokenTree> = tokens.into_iter().collect();
        for (index, token) in tokens.iter().enumerate() {
            match (
                token,
                tokens.get(index + 1),
                tokens.get(index + 2),
                tokens.get(index + 3),
            ) {
                (TokenTree::Group(group), ..) => self.read_tokens(group.stream()),
                (
                    TokenTree::Ident(owner),
                    Some(TokenTree::Punct(first)),
                    Some(TokenTree::Punct(second)),
                    Some(TokenTree::Ident(name)),
                ) if first.as_char() == ':' && second.as_char() == ':' => {
                    let segments = [owner.to_string(), name.to_string()];
                    self.read_path(&segments, owner.span().start().line, false);
                }
                _ => {}
            }
        }
    }
}

fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

/// Whether `attribute` can silence Clippy's disallowed methods, and if so
/// whether it is an `expect`. The lint is silenced by its own name, by the
/// `style` and `all` groups it belongs to, and by rustc's `warnings`.
fn suppresses_disallowed_methods(attribute: &syn::Attribute) -> Option<bool> {
    let kind = attribute.path().get_ident()?.to_string();
    if !matches!(kind.as_str(), "allow" | "expect" | "cfg_attr") {
        return None;
    }
    let syn::Meta::List(list) = &attribute.meta else {
        return None;
    };
    let tokens: Vec<String> = flatten(list.tokens.clone());
    let clippy_lint = tokens.windows(4).any(|window| {
        window[0] == "clippy"
            && window[1] == ":"
            && window[2] == ":"
            && matches!(window[3].as_str(), "disallowed_methods" | "all" | "style")
    });
    let warnings = tokens
        .iter()
        .enumerate()
        .any(|(index, token)| token == "warnings" && (index == 0 || tokens[index - 1] != ":"));
    let lint = clippy_lint || warnings;
    let expect = kind == "expect" || tokens.iter().any(|token| token == "expect");
    lint.then_some(expect)
}

fn flatten(tokens: TokenStream) -> Vec<String> {
    tokens
        .into_iter()
        .flat_map(|token| match token {
            TokenTree::Group(group) => flatten(group.stream()),
            other => vec![other.to_string()],
        })
        .collect()
}

impl<'ast> Visit<'ast> for ConstructionReader<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        if compiled_out_of_production(item_attributes(item)) {
            return;
        }
        let line = item.span().start().line;
        match item {
            Item::Impl(implementation) => {
                let name = self_type_name(&implementation.self_ty);
                if self.foreign(&name) {
                    self.refuse(
                        line,
                        format!("implements for sealed `{name}` outside its owner"),
                    );
                }
            }
            Item::Use(import) => self.read_use(&import.tree, line),
            // A file module's items are read as their own module, but the
            // attributes on its declaration belong to this one.
            Item::Mod(declaration) if declaration.content.is_none() => declaration
                .attrs
                .iter()
                .for_each(|attribute| self.visit_attribute(attribute)),
            Item::Type(alias) => {
                if let Type::Path(path) = alias.ty.as_ref() {
                    if let Some(last) = path.path.segments.last() {
                        if self.foreign(&last.ident.to_string()) {
                            self.refuse(
                                line,
                                format!("aliases sealed `{}` outside its owner", last.ident),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
        visit_item_in_scope(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        visit_trait_item_in_scope(self, item);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        visit_field_in_scope(self, field);
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        visit_variant_in_scope(self, variant);
    }

    /// A path in a pattern matches a value; it constructs nothing.
    fn visit_pat(&mut self, pattern: &'ast syn::Pat) {
        if !matches!(pattern, syn::Pat::Path(_)) {
            visit::visit_pat(self, pattern);
        }
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        let mut segments = path_segments(&expression.path);
        if let Some(qualified) = &expression.qself {
            segments.insert(qualified.position, self_type_name(&qualified.ty));
        }
        self.read_path(&segments, expression.span().start().line, false);
        visit::visit_expr_path(self, expression);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        let segments = path_segments(&expression.path);
        self.read_path(&segments, expression.span().start().line, true);
        visit::visit_expr_struct(self, expression);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        self.read_tokens(invocation.tokens.clone());
        visit::visit_macro(self, invocation);
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if let Some(expect) = suppresses_disallowed_methods(attribute) {
            let line = attribute.span().start().line;
            self.site(line, String::new(), SiteKind::Suppression { expect });
        }
    }
}

fn item_attributes(item: &Item) -> &[syn::Attribute] {
    match item {
        Item::Impl(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        _ => &[],
    }
}
