//! Machine-width observations have one owner across Execution and Relational.
//! Packet naming and name-to-work binding are sealed by the identity module;
//! this rule covers ambient observations that Rust signatures cannot exclude.
use super::production_cfg::compiled_out_of_production;
use super::production_scope::{production_graphs, production_modules};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::path::Path;
use syn::visit::{self, Visit};

const READER: &str = "crates/worth-relational/src/visibility/materialization/read_records/reader/";
const SCHEDULE_SOURCE: &str = "query_execution/leased_map.rs";
const SCHEDULE: &str = "schedule_query_packets";
const WIDTH_SOURCE: &str = "crates/worth-execution/src/authority/lease/worker_width.rs";
const WIDTH_READER: &str = "available_worker_count";

pub(super) fn validate(root: &Path) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for crate_root in ["crates/worth-execution", "crates/worth-relational"] {
        if !root.join(crate_root).exists() {
            continue;
        }
        let graphs = production_graphs(root, crate_root)?;
        for module in production_modules(&graphs) {
            for source in module.sources() {
                diagnostics.extend(
                    findings(&format!("{crate_root}/{source}"), module.items)
                        .into_iter()
                        .map(|(line, item, reason)| {
                            Diagnostic::new(
                                DiagnosticCode::Bc7011PacketIdentity,
                                format!("{crate_root}/{source}:{line}"),
                                format!("{item}: {reason}"),
                            )
                        }),
                );
            }
        }
    }
    Ok(diagnostics)
}

fn contains(tokens: TokenStream, names: &[&str]) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(name) => names
            .iter()
            .any(|s| name.to_string().eq_ignore_ascii_case(s)),
        TokenTree::Group(group) => contains(group.stream(), names),
        _ => false,
    })
}

fn machine_fact(tokens: TokenStream) -> bool {
    contains(tokens, &["available_parallelism", WIDTH_READER])
}

fn placement_fact(tokens: TokenStream) -> bool {
    contains(
        tokens,
        &[
            "available_parallelism",
            WIDTH_READER,
            "max_workers",
            "worker_count",
            "width",
        ],
    )
}

fn packetizes(source: &str) -> bool {
    let Some(relative) = source.strip_prefix(READER) else {
        return false;
    };
    ["query_packetization", "query_preparation_packetization"]
        .iter()
        .any(|name| relative == format!("{name}.rs") || relative.starts_with(&format!("{name}/")))
        || relative == SCHEDULE_SOURCE
        || relative.starts_with("query_execution/leased_map/")
}

fn mints_partition_identity(block: &syn::Block) -> bool {
    struct Constructor(bool);
    impl<'ast> Visit<'ast> for Constructor {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(function) = call.func.as_ref() {
                let mut segments = function.path.segments.iter().rev();
                if segments.next().is_some_and(|part| part.ident == "new")
                    && segments
                        .next()
                        .is_some_and(|part| part.ident == "PartitionIdentity")
                {
                    self.0 = true;
                }
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut constructor = Constructor(false);
    constructor.visit_block(block);
    constructor.0
}

fn partition_set_only(signature: &syn::Signature) -> bool {
    if signature.inputs.len() != 1
        || signature
            .generics
            .params
            .iter()
            .any(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
    {
        return false;
    }
    let Some(syn::FnArg::Typed(input)) = signature.inputs.first() else {
        return false;
    };
    let syn::Type::Path(ty) = input.ty.as_ref() else {
        return false;
    };
    let Some(segment) = ty.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    segment.ident == "Vec"
        && args.args.len() == 1
        && matches!(args.args.first(),Some(syn::GenericArgument::Type(syn::Type::Path(element))) if element.path.is_ident("PacketizedQueryWork"))
}

struct NamingBoundary<'a> {
    source: &'a str,
    found: Vec<(usize, String, &'static str)>,
    in_schedule: bool,
    in_width_home: bool,
    in_config: bool,
    depth: usize,
}

impl NamingBoundary<'_> {
    fn observes_placement(&self, tokens: TokenStream) -> bool {
        if self.source == format!("{READER}query_execution/leased_map/packet_identity.rs")
            && contains(tokens.clone(), &["ExecutionResourceLease", "lease"])
        {
            true
        } else if packetizes(self.source) {
            placement_fact(tokens)
        } else {
            machine_fact(tokens)
        }
    }

    fn function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let schedule = self.depth == 0
            && self.source == format!("{READER}{SCHEDULE_SOURCE}")
            && signature.ident == SCHEDULE;
        let width_home = self.depth == 0
            && self.in_config
            && self.source == WIDTH_SOURCE
            && signature.ident == WIDTH_READER;
        let naming = self.source.ends_with("leased_map/packet_identity.rs")
            && signature.ident == "identities";
        let reason = if naming && !partition_set_only(signature) {
            Some("packet naming accepts only the canonical partition set")
        } else if self.source == format!("{READER}query_execution/leased_map/packet_identity.rs")
            && contains(
                quote::quote!(#signature #block),
                &["ExecutionResourceLease", "lease"],
            )
        {
            Some("packet declaration finishes before scheduling receives the lease")
        } else if self.source == format!("{READER}{SCHEDULE_SOURCE}")
            && mints_partition_identity(block)
        {
            Some("packet identities are minted only by their declaration owner")
        } else if schedule {
            machine_fact(quote::quote!(#signature #block))
                .then_some("machine observations belong only to the authority width reader")
        } else if width_home {
            None
        } else {
            self.observes_placement(quote::quote!(#signature #block)).then_some("machine observations belong to the authority width reader; packet placement belongs to the schedule")
        };
        if let Some(reason) = reason {
            self.found.push((
                signature.ident.span().start().line,
                signature.ident.to_string(),
                reason,
            ));
        }
        let parent = (self.in_schedule, self.in_width_home);
        self.in_schedule = schedule;
        self.in_width_home = width_home;
        self.depth += 1;
        visit::visit_block(self, block);
        self.depth -= 1;
        (self.in_schedule, self.in_width_home) = parent;
    }
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
            syn::Item::Fn(_) | syn::Item::Mod(_) | syn::Item::Use(_) | syn::Item::Impl(_)
        ) && !self.in_schedule
            && !self.in_width_home
            && self.observes_placement(item.to_token_stream())
        {
            use syn::spanned::Spanned;
            self.found.push((item.span().start().line,"item".to_owned(),"machine observations belong to the authority width reader; packet placement belongs to the schedule"));
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if !compiled_out_of_production(&function.attrs) {
            self.function(&function.sig, &function.block);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if compiled_out_of_production(&item.attrs) {
            return;
        }
        let parent = self.in_config;
        self.in_config = matches!(item.self_ty.as_ref(),syn::Type::Path(p) if p.path.is_ident("ExecutionAuthorityConfig"));
        visit::visit_item_impl(self, item);
        self.in_config = parent;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        if !compiled_out_of_production(&function.attrs) {
            self.function(&function.sig, &function.block);
        }
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if !self.in_schedule
            && !self.in_width_home
            && self.observes_placement(item.to_token_stream())
        {
            self.found.push((
                item.use_token.span.start().line,
                "use".to_owned(),
                "import machine observations only inside the authority width reader or schedule",
            ));
        }
    }
}

fn findings(source: &str, items: &[syn::Item]) -> Vec<(usize, String, &'static str)> {
    let mut visitor = NamingBoundary {
        source,
        found: Vec::new(),
        in_schedule: false,
        in_width_home: false,
        in_config: false,
        depth: 0,
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found
}

#[cfg(test)]
mod compiler;
#[cfg(test)]
mod tests;
