//! Distinguish raw allocation patterns from lawful constructor expressions.
use super::super::production_cfg::compiled_out_of_production;
use super::super::production_scope::{
    visit_field_in_scope, visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope,
    visit_variant_in_scope, ScopedVisitor,
};
use super::super::use_binding_resolution::expand_use_tree;
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, Pat, Token, TraitItem};

const RAW: &str = "ExecutionAllocationDenialKind";

pub(super) fn findings(items: &[Item]) -> BTreeSet<(String, usize)> {
    let mut visitor = RawPatterns {
        scope: Vec::new(),
        aliases: BTreeSet::from([RAW.to_owned()]),
        found: BTreeSet::new(),
    };
    visitor.items(items);
    visitor.found
}

struct RawPatterns {
    scope: Vec<String>,
    aliases: BTreeSet<String>,
    found: BTreeSet<(String, usize)>,
}

impl ScopedVisitor for RawPatterns {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl RawPatterns {
    fn import(&mut self, item: &syn::ItemUse) {
        if compiled_out_of_production(&item.attrs) {
            return;
        }
        for (_, name, binding) in expand_use_tree(&[], &item.tree) {
            if name == RAW {
                self.aliases.insert(binding);
            }
        }
    }

    fn items(&mut self, items: &[Item]) {
        for item in items {
            if let Item::Use(import) = item {
                self.import(import);
            }
        }
        for item in items {
            self.visit_item(item);
        }
    }

    fn pattern(&mut self, pattern: &Pat) {
        let mut visitor = PatternPaths {
            aliases: &self.aliases,
            lines: BTreeSet::new(),
        };
        visitor.visit_pat(pattern);
        self.found.extend(
            visitor
                .lines
                .into_iter()
                .map(|line| (self.scope.join("::"), line)),
        );
    }

    fn matches(&mut self, tokens: TokenStream) {
        if let Ok(input) = syn::parse2::<MatchesInput>(tokens.clone()) {
            self.pattern(&input.pattern);
        } else {
            // A macro metavariable can stand in the value expression. The
            // first outer comma still separates it from the pattern.
            let pattern: TokenStream = tokens
                .into_iter()
                .skip_while(|token| !matches!(token, TokenTree::Punct(p) if p.as_char() == ','))
                .skip(1)
                .collect();
            self.raw_pattern_tokens(pattern);
        }
    }

    fn raw_pattern_tokens(&mut self, tokens: TokenStream) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for window in tokens.windows(4) {
            if let [TokenTree::Ident(name), TokenTree::Punct(first), TokenTree::Punct(second), TokenTree::Ident(_)] =
                window
            {
                if self.aliases.contains(&name.to_string())
                    && first.as_char() == ':'
                    && second.as_char() == ':'
                {
                    self.found
                        .insert((self.scope.join("::"), name.span().start().line));
                }
            }
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.raw_pattern_tokens(group.stream());
            }
        }
    }

    fn nested_macros(&mut self, tokens: TokenStream) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for window in tokens.windows(3) {
            if let [TokenTree::Ident(name), TokenTree::Punct(bang), TokenTree::Group(group)] =
                window
            {
                if name == "matches" && bang.as_char() == '!' {
                    self.matches(group.stream());
                }
            }
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.nested_macros(group.stream());
            }
        }
    }
}

impl<'ast> Visit<'ast> for RawPatterns {
    fn visit_item(&mut self, item: &'ast Item) {
        let aliases = self.aliases.clone();
        if let Item::Mod(module) = item {
            if let Some((_, items)) = &module.content {
                for item in items {
                    if let Item::Use(import) = item {
                        self.import(import);
                    }
                }
            }
        }
        visit_item_in_scope(self, item);
        self.aliases = aliases;
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.import(item);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let aliases = self.aliases.clone();
        for statement in &block.stmts {
            if let syn::Stmt::Item(Item::Use(import)) = statement {
                self.import(import);
            }
        }
        visit::visit_block(self, block);
        self.aliases = aliases;
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
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        if compiled_out_of_production(&arm.attrs) {
            return;
        }
        self.pattern(&arm.pat);
        visit::visit_arm(self, arm);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "matches")
        {
            self.matches(mac.tokens.clone());
        }
        self.nested_macros(mac.tokens.clone());
    }
}

struct PatternPaths<'a> {
    aliases: &'a BTreeSet<String>,
    lines: BTreeSet<usize>,
}
impl<'ast> Visit<'ast> for PatternPaths<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<_> = path.segments.iter().collect();
        if segments.len() >= 2
            && self
                .aliases
                .contains(&segments[segments.len() - 2].ident.to_string())
        {
            self.lines.insert(path.span().start().line);
        }
        visit::visit_path(self, path);
    }
}

struct MatchesInput {
    pattern: Pat,
}
impl Parse for MatchesInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: Expr = input.parse()?;
        let _: Token![,] = input.parse()?;
        let pattern = Pat::parse_multi_with_leading_vert(input)?;
        if input.peek(Token![if]) {
            let _: Token![if] = input.parse()?;
            let _: Expr = input.parse()?;
        }
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
        }
        Ok(Self { pattern })
    }
}
