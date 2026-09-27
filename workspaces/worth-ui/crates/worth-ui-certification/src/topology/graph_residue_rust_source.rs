use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Arm, Expr, ExprMethodCall, ExprPath, File, Ident, ImplItem, Item, Macro, UseName};

use super::WorkspaceSourceInventory;

pub(super) fn collect_method_names(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> Vec<String> {
    let parsed = parse_rust_file(inventory, path);
    let mut collector = MethodCallCollector::default();
    collector.visit_file(&parsed);
    collector.method_names
}

pub(super) fn collect_method_names_for_function(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
    function_name: &str,
) -> Vec<String> {
    let mut collector = MethodCallCollector::default();
    visit_function(
        &parse_rust_file(inventory, path),
        function_name,
        &mut collector,
    );
    collector.method_names
}

pub(super) fn collect_paths_for_function(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
    function_name: &str,
) -> Vec<Vec<String>> {
    let mut collector = PathCollector::default();
    visit_function(
        &parse_rust_file(inventory, path),
        function_name,
        &mut collector,
    );
    collector.paths
}

/// Whether the file names `variant` anywhere but in the pattern of an arm
/// that yields `None` or a plain `use` of it. Every other mention, from an
/// accepting arm, `if let`, `let`-`else`, a macro, or a renamed import, can
/// let the variant through.
pub(super) fn file_accepts_match_variant(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
    variant: &str,
) -> bool {
    parsed_file_accepts_match_variant(&parse_rust_file(inventory, path), variant)
}

pub(super) fn ends_with_path(segments: &[String], suffix: &[&str]) -> bool {
    segments.len() >= suffix.len()
        && segments[segments.len() - suffix.len()..]
            .iter()
            .map(String::as_str)
            .eq(suffix.iter().copied())
}

fn parsed_file_accepts_match_variant(parsed: &File, variant: &str) -> bool {
    let mut mentions = VariantMentionCollector::new(variant);
    mentions.visit_file(parsed);
    mentions.mentions > mentions.refused
}

fn visit_function<'ast>(parsed: &'ast File, function_name: &str, visitor: &mut impl Visit<'ast>) {
    for item in &parsed.items {
        match item {
            Item::Fn(item_fn) if item_fn.sig.ident == function_name => {
                visitor.visit_block(&item_fn.block);
            }
            Item::Impl(item_impl) => {
                for impl_item in &item_impl.items {
                    if let ImplItem::Fn(function) = impl_item {
                        if function.sig.ident == function_name {
                            visitor.visit_block(&function.block);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn parse_rust_file(inventory: &WorkspaceSourceInventory, path: &Path) -> File {
    let text = inventory.text(path);
    syn::parse_file(text).unwrap_or_else(|error| {
        panic!("{} should parse as Rust source: {error}", path.display());
    })
}

fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

#[derive(Default)]
struct MethodCallCollector {
    method_names: Vec<String>,
}

impl Visit<'_> for MethodCallCollector {
    fn visit_expr_method_call(&mut self, method_call: &ExprMethodCall) {
        self.method_names.push(method_call.method.to_string());
        visit::visit_expr_method_call(self, method_call);
    }
}

#[derive(Default)]
struct PathCollector {
    paths: Vec<Vec<String>>,
}

impl Visit<'_> for PathCollector {
    fn visit_expr_path(&mut self, expr_path: &ExprPath) {
        self.paths.push(path_segments(&expr_path.path));
        visit::visit_expr_path(self, expr_path);
    }
}

struct VariantMentionCollector<'variant> {
    variant: &'variant str,
    mentions: usize,
    refused: usize,
}

impl<'variant> VariantMentionCollector<'variant> {
    const fn new(variant: &'variant str) -> Self {
        Self {
            variant,
            mentions: 0,
            refused: 0,
        }
    }
}

impl Visit<'_> for VariantMentionCollector<'_> {
    fn visit_ident(&mut self, ident: &Ident) {
        if ident == self.variant {
            self.mentions += 1;
        }
    }

    fn visit_macro(&mut self, mac: &Macro) {
        // syn leaves a macro body as tokens, so read its words directly.
        let body = mac.tokens.to_string();
        self.mentions += body
            .split(|character: char| !(character.is_alphanumeric() || character == '_'))
            .filter(|word| *word == self.variant)
            .count();
        visit::visit_macro(self, mac);
    }

    fn visit_arm(&mut self, arm: &Arm) {
        if yields_none(&arm.body) {
            let mut pattern = VariantMentionCollector::new(self.variant);
            pattern.visit_pat(&arm.pat);
            self.refused += pattern.mentions;
        }
        visit::visit_arm(self, arm);
    }

    fn visit_use_name(&mut self, name: &UseName) {
        if name.ident == self.variant {
            self.refused += 1;
        }
        visit::visit_use_name(self, name);
    }
}

fn yields_none(body: &Expr) -> bool {
    matches!(body, Expr::Path(expr_path) if expr_path.path.is_ident("None"))
}

#[cfg(test)]
mod tests {
    fn accepts(source: &str) -> bool {
        let parsed = syn::parse_file(source).expect("test source parses");
        super::parsed_file_accepts_match_variant(&parsed, "Node")
    }

    #[test]
    fn a_refusing_arm_may_name_the_variant() {
        assert!(!accepts(
            "fn route(t: Target) -> Option<u8> { match t { Target::Root | Target::Node { .. } => None, Target::Leaf => Some(1) } }"
        ));
    }

    #[test]
    fn an_arm_that_answers_for_the_variant_accepts_it() {
        for source in [
            "fn route(t: Target) -> Option<u8> { match t { Target::Node { digest } => lookup(digest), _ => None } }",
            "fn route(t: Target) -> Option<u8> { match t { Target::Root | Target::Node(_) => Some(0), _ => None } }",
            "impl App { fn route(&self, t: Target) -> Option<u8> { match t { Target::Node { .. } => Some(2), _ => None } } }",
        ] {
            assert!(accepts(source), "{source}");
        }
    }

    #[test]
    fn every_function_in_the_file_is_read() {
        assert!(accepts(
            "fn route(t: Target) -> Option<u8> { match t { Target::Node { .. } => None, _ => None } }
             fn renamed(t: Target) -> Option<u8> { match t { Target::Node { digest } => lookup(digest), _ => None } }"
        ));
    }

    #[test]
    fn naming_the_variant_outside_a_match_arm_accepts_it() {
        for source in [
            "fn route(t: Target) -> Option<u8> { if let Target::Node { digest } = t { return lookup(digest); } None }",
            "fn route(t: Target) -> Option<u8> { let Target::Node { digest } = t else { return None }; lookup(digest) }",
            "fn route(t: Target) -> Option<u8> { matches!(t, Target::Node { .. }).then_some(0) }",
            "fn route(t: Target) -> Option<u8> { match t { Target::Node { .. } => None, _ => None }; lookup_by(Target::Node) }",
        ] {
            assert!(accepts(source), "{source}");
        }
    }

    #[test]
    fn importing_the_variant_under_another_name_accepts_it() {
        for source in [
            "use crate::Target::Node as Leaf; fn route(t: Target) -> Option<u8> { match t { Leaf { digest } => lookup(digest), _ => None } }",
            "use crate::{Target, Target::Node as Leaf}; fn route() {}",
        ] {
            assert!(accepts(source), "{source}");
        }
        assert!(!accepts(
            "use crate::Target::{self, Node}; fn route(t: Target) -> Option<u8> { match t { Node { .. } => None, _ => None } }"
        ));
    }
}
