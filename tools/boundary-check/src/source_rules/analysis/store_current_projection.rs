//! One current recovery-projection wire grammar; operations cannot select it.

#[cfg(test)]
mod tests;
mod wire_boundary;

use super::crate_modules::{GovernedCrate, ModuleGraph};
use super::production_cfg::{compiled_out_of_production, module_compiled_out_of_production};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, Lit, Type};

const DOMAIN_PREFIX: &[u8] = b"store.physical.recovery-projection.v";
const CURRENT_DOMAIN: &str = "CURRENT_RECOVERY_PROJECTION_DOMAIN";
const PROJECTION: &str = "PersistedPhysicalRecoveryProjection";

pub(super) fn enforce(governed: &GovernedCrate, graph: &ModuleGraph) -> Vec<Diagnostic> {
    if governed.package != "worth-store-physical-format" {
        return Vec::new();
    }
    let mut rule = CurrentProjection::default();
    for (path, node) in &graph.modules {
        if module_compiled_out_of_production(graph, path) {
            continue;
        }
        rule.projection_scope = path
            .first()
            .is_some_and(|name| name == "recovery_projection");
        for item in &node.items {
            rule.visit_item(item);
        }
    }
    rule.finish()
        .into_iter()
        .map(|finding| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                format!("{}::recovery_projection", governed.package),
                format!("Store current-only projection boundary: {finding}"),
            )
        })
        .collect()
}

#[derive(Default)]
struct CurrentProjection {
    projection_scope: bool,
    domains: usize,
    current_declarations: usize,
    projections: usize,
    encoders: usize,
    decoders: usize,
    admissions: usize,
    findings: Vec<String>,
}

impl CurrentProjection {
    fn finish(mut self) -> Vec<String> {
        for (count, responsibility) in [
            (self.domains, "supported projection domain literal"),
            (
                self.current_declarations,
                "fixed current-domain declaration",
            ),
            (self.projections, "sealed projection declaration"),
            (self.encoders, "projection encoder"),
            (self.decoders, "projection decoder"),
            (self.admissions, "current-domain admission function"),
        ] {
            if count != 1 {
                self.findings.push(format!(
                    "expected exactly one {responsibility}, found {count}"
                ));
            }
        }
        self.findings.sort();
        self.findings.dedup();
        self.findings
    }

    fn domain_literal(&mut self, literal: &Lit) {
        let bytes = match literal {
            Lit::ByteStr(value) => value.value(),
            Lit::Str(value) => value.value().into_bytes(),
            _ => return,
        };
        if bytes
            .strip_prefix(DOMAIN_PREFIX)
            .is_some_and(|suffix| !suffix.is_empty() && suffix.iter().all(u8::is_ascii_digit))
        {
            self.domains += 1;
        }
    }

    fn projection_fields(&mut self, item: &syn::ItemStruct) {
        self.projections += 1;
        let fields = [
            "source_root_generation",
            "root_state",
            "record_identities",
            "payload",
            "operation",
            "placements",
            "segment_updates",
            "manifests",
        ];
        for field in &item.fields {
            let name = field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            if !fields.contains(&name.as_str()) || !matches!(field.vis, syn::Visibility::Inherited)
            {
                self.findings.push("projection fields must remain sealed common state plus one typed operation; wire selectors and parallel attachments are forbidden".into());
            }
        }
        if !item.fields.iter().any(|field| {
            field.ident.as_ref().is_some_and(|name| name == "operation")
                && matches!(&field.ty, Type::Path(path) if path.path.is_ident("PersistedPhysicalRecoveryOperation"))
        }) {
            self.findings.push("projection must store the complete typed operation".into());
        }
    }
}

impl<'ast> Visit<'ast> for CurrentProjection {
    fn visit_item(&mut self, item: &'ast Item) {
        let attributes = match item {
            Item::Const(item) => &item.attrs,
            Item::Enum(item) => &item.attrs,
            Item::Fn(item) => &item.attrs,
            Item::Impl(item) => &item.attrs,
            Item::Macro(item) => &item.attrs,
            Item::Mod(item) => &item.attrs,
            Item::Struct(item) => &item.attrs,
            _ => return visit::visit_item(self, item),
        };
        if !compiled_out_of_production(attributes) {
            visit::visit_item(self, item);
        }
    }

    // ModuleGraph already visits each inline/file module and its cfg posture.
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}

    fn visit_expr_lit(&mut self, expression: &'ast syn::ExprLit) {
        self.domain_literal(&expression.lit);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if item.ident == CURRENT_DOMAIN {
            self.current_declarations += 1;
            if !matches!(item.expr.as_ref(), Expr::Lit(value) if matches!(&value.lit, Lit::ByteStr(bytes) if bytes.value().strip_prefix(DOMAIN_PREFIX).is_some_and(|suffix| !suffix.is_empty() && suffix.iter().all(u8::is_ascii_digit))))
            {
                self.findings.push("current domain must be one fixed byte-string identity, not a computed selection".into());
            }
        }
        visit::visit_item_const(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if item.ident == PROJECTION {
            self.projection_fields(item);
        }
        visit::visit_item_struct(self, item);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        if self.projection_scope {
            for argument in &signature.inputs {
                if let syn::FnArg::Typed(argument) = argument {
                    if matches!(argument.pat.as_ref(), syn::Pat::Ident(name) if ["version", "schema", "wire_version", "format_version"].contains(&name.ident.to_string().as_str()))
                    {
                        self.findings
                            .push("operation/codec inputs may not select a wire version".into());
                    }
                }
            }
        }
        visit::visit_signature(self, signature);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if matches!(item.self_ty.as_ref(), Type::Path(path) if path.path.segments.last().is_some_and(|part| part.ident == PROJECTION))
        {
            for method in &item.items {
                let ImplItem::Fn(method) = method else {
                    continue;
                };
                if compiled_out_of_production(&method.attrs) {
                    continue;
                }
                if method.sig.ident == "encode" {
                    self.encoders += 1;
                    if !wire_boundary::fixed_encoder(&method.block) {
                        self.findings.push("projection encoder must write the fixed current domain unconditionally before operation data".into());
                    }
                }
                if method.sig.ident == "decode_payload" {
                    self.decoders += 1;
                    if !wire_boundary::early_admission(&method.block) {
                        self.findings.push("projection decoder must admit the current domain before body decoding/allocation".into());
                    }
                }
            }
        }
        visit::visit_item_impl(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if item.sig.ident == "require_current_domain" {
            self.admissions += 1;
            if !wire_boundary::sole_current_success(&item.block) {
                self.findings.push("domain admission must have exactly one success: equality with the fixed current domain; all other domains deny".into());
            }
        }
        visit::visit_item_fn(self, item);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        fn literals(rule: &mut CurrentProjection, stream: proc_macro2::TokenStream) {
            for token in stream {
                match token {
                    proc_macro2::TokenTree::Literal(value) => {
                        if let Ok(literal) = syn::parse_str::<Lit>(&value.to_string()) {
                            rule.domain_literal(&literal);
                        }
                    }
                    proc_macro2::TokenTree::Group(group) => literals(rule, group.stream()),
                    _ => {}
                }
            }
        }
        literals(self, item.tokens.clone());
    }
}
