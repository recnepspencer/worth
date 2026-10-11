//! Packet naming cannot observe placement; scheduling cannot mint identities.
use super::production_cfg::compiled_out_of_production;
use super::production_scope::{production_graphs, production_modules};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::path::Path;
use syn::visit::{self, Visit};

const READER: &str = "src/visibility/materialization/read_records/reader/";
const SCHEDULE_SOURCE: &str = "query_execution/leased_map.rs";
const SCHEDULE: &str = "schedule_query_packets";

pub(super) fn validate(root: &Path) -> Result<Vec<Diagnostic>, String> {
    let crate_root = "crates/worth-relational";
    if !root.join(crate_root).exists() {
        return Ok(Vec::new());
    }
    let graphs = production_graphs(root, crate_root)?;
    let mut diagnostics = Vec::new();
    for module in production_modules(&graphs) {
        for source in module.sources().filter(|source| packetizes(source)) {
            diagnostics.extend(findings(source, module.items).into_iter().map(
                |(line, item, reason)| {
                    Diagnostic::new(
                        DiagnosticCode::Bc7011PacketIdentity,
                        format!("{crate_root}/{source}:{line}"),
                        format!("{item}: {reason}"),
                    )
                },
            ));
        }
    }
    Ok(diagnostics)
}

fn packetizes(source: &str) -> bool {
    source.strip_prefix(READER).is_some_and(|source| {
        source == "query_packetization.rs"
            || source.starts_with("query_packetization/")
            || source == SCHEDULE_SOURCE
            || source.starts_with("query_execution/leased_map/")
    })
}

fn contains(tokens: TokenStream, names: &[&str]) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(name) => names.iter().any(|spelling| name == spelling),
        TokenTree::Group(group) => contains(group.stream(), names),
        _ => false,
    })
}

fn placement_fact(tokens: TokenStream) -> bool {
    contains(
        tokens,
        &[
            "available_parallelism",
            "max_workers",
            "worker_count",
            "width",
        ],
    )
}

struct NamingBoundary<'a> {
    source: &'a str,
    found: Vec<(usize, String, &'static str)>,
}

impl<'ast> Visit<'ast> for NamingBoundary<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if let syn::Item::Mod(module) = item {
            if compiled_out_of_production(&module.attrs) {
                return;
            }
        }
        visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if compiled_out_of_production(&function.attrs) {
            return;
        }
        let schedule =
            self.source == format!("{READER}{SCHEDULE_SOURCE}") && function.sig.ident == SCHEDULE;
        let reason = if schedule {
            contains(function.block.to_token_stream(), &["PartitionIdentity"])
                .then_some("the schedule consumes identities; it cannot mint packet names")
        } else {
            placement_fact(function.to_token_stream())
                .then_some("placement facts belong only to the packet schedule")
        };
        if let Some(reason) = reason {
            self.found.push((
                function.sig.ident.span().start().line,
                function.sig.ident.to_string(),
                reason,
            ));
        }
        // Nested functions do not inherit the schedule's permission.
        visit::visit_block(self, &function.block);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if placement_fact(item.to_token_stream()) {
            self.found.push((
                item.use_token.span.start().line,
                "use".to_owned(),
                "import machine facts inside the packet schedule only",
            ));
        }
    }
}

fn findings(source: &str, items: &[syn::Item]) -> Vec<(usize, String, &'static str)> {
    let mut visitor = NamingBoundary {
        source,
        found: Vec::new(),
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found
}

#[cfg(test)]
mod tests;
