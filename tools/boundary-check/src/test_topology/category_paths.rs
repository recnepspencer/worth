//! Categories are top-level path prefixes; behaviour names carry no taxonomy.
use super::failure;
use crate::{
    diagnostics::Diagnostic,
    source_rules::{parse_compiled_target_where, GovernedCrate},
};
use std::{collections::BTreeSet, path::Path};
use syn::parse::Parser;
use syn::visit::{self, Visit};

pub(super) const CATEGORIES: &[&str] =
    &["process", "oracle", "cost", "determinism", "compile_fail"];
const RETIRED: &[&str] = &["fast", "exhaustive", "scale", "gpu"];

pub(super) fn validate(owner: &GovernedCrate, target: &Path) -> Vec<Diagnostic> {
    let graph = match parse_compiled_target_where(owner, target, &|_| true) {
        Ok(graph) => graph,
        Err(error) => return vec![failure(&owner.relative_crate_root, error)],
    };
    let mut diagnostics = Vec::new();
    let mut identities = BTreeSet::new();
    for (path, node) in graph.modules {
        let segments: Vec<_> = path
            .iter()
            .map(|p| p.strip_prefix("r#").unwrap_or(p))
            .collect();
        if segments.iter().any(|p| RETIRED.contains(p))
            || segments.iter().skip(1).any(|p| CATEGORIES.contains(p))
            || (segments.contains(&"serial")
                && (!matches!(segments.first(), Some(&"process" | &"compile_fail"))
                    || segments.get(1) != Some(&"serial")))
        {
            diagnostics.push(failure(
                &owner.relative_crate_root,
                format!(
                    "invalid category path `{}`; categories must be top-level prefixes",
                    path.join("::")
                ),
            ));
        }
        let mut tests = Tests::default();
        for item in &node.items {
            if !matches!(item, syn::Item::Mod(_)) {
                tests.visit_item(item);
            }
        }
        for name in tests.names {
            if carries_id(&name) || name.starts_with("test_") || name.starts_with("it_") {
                diagnostics.push(failure(&format!("{}/{}", owner.relative_crate_root, node.relative_source), format!("test `{name}` must name its behaviour without an identifier or test_/it_ prefix")));
            }
            if !identities.insert((path.clone(), name.clone())) {
                diagnostics.push(failure(
                    &owner.relative_crate_root,
                    format!("test behaviour `{name}` is not unique within its court"),
                ));
            }
        }
    }
    diagnostics
}

pub(super) fn carries_id(name: &str) -> bool {
    let parts: Vec<_> = name.split('_').collect();
    parts.iter().enumerate().any(|(i, part)| {
        let numbered = [
            "m",
            "d",
            "r",
            "milestone",
            "phase",
            "decision",
            "ticket",
            "req",
        ]
        .iter()
        .any(|prefix| {
            part.strip_prefix(prefix)
                .is_some_and(|tail| !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()))
        });
        let phase = ["milestone", "phase", "decision", "ticket"].contains(part)
            && parts.get(i + 1).is_some_and(|next| {
                next.bytes().all(|b| b.is_ascii_digit())
                    || [
                        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
                        "ten",
                    ]
                    .contains(next)
            });
        numbered || phase
    })
}

#[derive(Default)]
struct Tests {
    names: Vec<String>,
}
impl<'ast> Visit<'ast> for Tests {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if function.attrs.iter().any(|a| test_attribute(&a.meta)) {
            self.names.push(function.sig.ident.to_string());
        }
        visit::visit_item_fn(self, function);
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // Expand parsable item bodies, including macros wrapping test functions.
        if let Ok(file) = syn::parse2::<syn::File>(item.mac.tokens.clone()) {
            self.visit_file(&file);
        }
    }
}

pub(super) fn test_attribute(meta: &syn::Meta) -> bool {
    if meta
        .path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test")
    {
        return true;
    }
    let syn::Meta::List(list) = meta else {
        return false;
    };
    list.path.is_ident("cfg_attr")
        && syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .is_ok_and(|metas| metas.iter().skip(1).any(test_attribute))
}
