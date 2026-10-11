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
        TokenTree::Ident(name) => names
            .iter()
            .any(|spelling| name.to_string().eq_ignore_ascii_case(spelling)),
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

fn partition_set_only(signature: &syn::Signature) -> bool {
    if signature.inputs.len() != 1
        || signature
            .generics
            .params
            .iter()
            .any(|parameter| !matches!(parameter, syn::GenericParam::Lifetime(_)))
    {
        return false;
    }
    let Some(syn::FnArg::Typed(input)) = signature.inputs.first() else {
        return false;
    };
    let syn::Type::Reference(reference) = input.ty.as_ref() else {
        return false;
    };
    let syn::Type::Slice(slice) = reference.elem.as_ref() else {
        return false;
    };
    let syn::Type::Path(element) = slice.elem.as_ref() else {
        return false;
    };
    reference.mutability.is_none() && element.path.is_ident("PacketizedQueryWork")
}

struct NamingBoundary<'a> {
    source: &'a str,
    found: Vec<(usize, String, &'static str)>,
    in_schedule: bool,
}

impl<'ast> Visit<'ast> for NamingBoundary<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if let syn::Item::Mod(module) = item {
            if compiled_out_of_production(&module.attrs) {
                return;
            }
        }
        if !matches!(
            item,
            syn::Item::Fn(_) | syn::Item::Mod(_) | syn::Item::Use(_)
        ) && !self.in_schedule
            && placement_fact(item.to_token_stream())
        {
            use syn::spanned::Spanned;
            self.found.push((
                item.span().start().line,
                "item".to_owned(),
                "placement facts belong only to the packet schedule",
            ));
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if compiled_out_of_production(&function.attrs) {
            return;
        }
        let schedule =
            self.source == format!("{READER}{SCHEDULE_SOURCE}") && function.sig.ident == SCHEDULE;
        let naming = self
            .source
            .ends_with("query_execution/leased_map/packet_identity.rs")
            && function.sig.ident == "identities";
        let reason = if naming && !partition_set_only(&function.sig) {
            Some("packet naming accepts only the immutable canonical partition set")
        } else if schedule {
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
        let parent = self.in_schedule;
        self.in_schedule = schedule;
        visit::visit_block(self, &function.block);
        self.in_schedule = parent;
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if !self.in_schedule && placement_fact(item.to_token_stream()) {
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
        in_schedule: false,
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found
}

#[cfg(test)]
mod tests;
