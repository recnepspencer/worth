//! Fresh def-site (`mixed_site`) identifiers for `#[derive(WorthBinaryValue)]`.
use super::grammar::{BinaryValueDeclaration, StructShape};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{DeriveInput, Ident};

pub(super) struct HygienicNames {
    pub(super) value_alias: Ident,
    pub(super) binding_alias: Ident,
    pub(super) delegate_struct: Ident,
    pub(super) wire_bytes_const: Ident,
    pub(super) field_aliases: Vec<Ident>,
    pub(super) field_locals: Vec<Ident>,
    pub(super) call_validate: Ident,
    pub(super) value_param: Ident,
    pub(super) writer_param: Ident,
    pub(super) bytes_param: Ident,
    pub(super) admission_param: Ident,
    pub(super) raw_param: Ident,
    pub(super) rest_local: Ident,
    pub(super) observed_local: Ident,
    pub(super) slice_local: Ident,
    pub(super) decoded_local: Ident,
    pub(super) reason_param: Ident,
}

impl HygienicNames {
    pub(super) fn new(input: &DeriveInput, spec: &BinaryValueDeclaration) -> Self {
        let mut used = collect_used_idents(input, spec);
        let mixed = Span::mixed_site();
        let field_count = spec.field_types().len();
        let field_aliases = (0..field_count)
            .map(|i| fresh_ident(&format!("__WorthBinaryField{i}"), mixed, &mut used))
            .collect();
        let field_locals = (0..field_count)
            .map(|i| fresh_ident(&format!("__worth_binary_f{i}"), mixed, &mut used))
            .collect();
        Self {
            value_alias: fresh_ident("__WorthBinaryValue", mixed, &mut used),
            binding_alias: fresh_ident("__WorthBinaryBinding", mixed, &mut used),
            delegate_struct: fresh_ident("__WorthBinaryDelegate", mixed, &mut used),
            wire_bytes_const: fresh_ident("__WORTH_BINARY_WIRE_BYTES", mixed, &mut used),
            field_aliases,
            field_locals,
            call_validate: fresh_ident("__worth_binary_call_validate", mixed, &mut used),
            value_param: fresh_ident("__worth_binary_value", mixed, &mut used),
            writer_param: fresh_ident("__worth_binary_writer", mixed, &mut used),
            bytes_param: fresh_ident("__worth_binary_bytes", mixed, &mut used),
            admission_param: fresh_ident("__worth_binary_admission", mixed, &mut used),
            raw_param: fresh_ident("__worth_binary_raw", mixed, &mut used),
            rest_local: fresh_ident("__worth_binary_rest", mixed, &mut used),
            observed_local: fresh_ident("__worth_binary_observed", mixed, &mut used),
            slice_local: fresh_ident("__worth_binary_slice", mixed, &mut used),
            decoded_local: fresh_ident("__worth_binary_decoded", mixed, &mut used),
            reason_param: fresh_ident("__worth_binary_reason", mixed, &mut used),
        }
    }
}

fn collect_used_idents(input: &DeriveInput, spec: &BinaryValueDeclaration) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_stream_idents(input.ident.to_token_stream(), &mut used);
    collect_stream_idents(spec.binding.to_token_stream(), &mut used);
    if let Some(validate) = &spec.validate {
        collect_stream_idents(validate.to_token_stream(), &mut used);
    }
    if let StructShape::Named(fields) = &spec.shape {
        for field in fields {
            collect_stream_idents(field.ident.to_token_stream(), &mut used);
        }
    }
    for ty in spec.field_types() {
        collect_stream_idents(ty.to_token_stream(), &mut used);
    }
    used
}

fn collect_stream_idents(stream: TokenStream, out: &mut BTreeSet<String>) {
    for tree in stream {
        match tree {
            TokenTree::Ident(ident) => {
                let text = ident.to_string();
                let normalized = text.strip_prefix("r#").unwrap_or(&text);
                out.insert(normalized.to_owned());
            }
            TokenTree::Group(group) => collect_stream_idents(group.stream(), out),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn fresh_ident(base: &str, span: Span, used: &mut BTreeSet<String>) -> Ident {
    if used.insert(base.to_owned()) {
        return Ident::new(base, span);
    }
    let mut suffix = 1usize;
    loop {
        let candidate = format!("{base}_{suffix}");
        if used.insert(candidate.clone()) {
            return Ident::new(&candidate, span);
        }
        suffix += 1;
    }
}
