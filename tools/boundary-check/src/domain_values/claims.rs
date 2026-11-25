//! Discover actual binding roots, including aliases of the declaration macro.
use super::binary_derive_claims::{
    collect_binary_aliases, derived_binary_binding_name, inspect_worth_binary_derive,
};
use super::canonical_claims::canonical_binding;
use super::modules::Sources;
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::{
    parse::{Parse, ParseStream},
    visit::Visit,
    ItemImpl, Macro, Type,
};

#[derive(Clone)]
pub(super) struct Claim {
    pub source: String,
    pub modules: Vec<String>,
    pub binding: String,
    pub value: Type,
    pub origin: Option<syn::Path>,
}

struct Declaration {
    binding: String,
    value: Type,
    body: TokenStream,
}
impl Parse for Declaration {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: syn::Visibility = input.parse()?;
        let binding: syn::Ident = input.parse()?;
        input.parse::<syn::Token![for]>()?;
        let value = input.parse()?;
        let body;
        syn::braced!(body in input);
        Ok(Self {
            binding: binding.to_string(),
            value,
            body: body.parse()?,
        })
    }
}

fn marked(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(id) => [
            "binary",
            "domain_value",
            "DOMAIN_VALUE",
            "ApplicationCanonicalValueContract",
            "canonical_contract",
            "WorthBinaryValue",
            "worth_binary",
        ]
        .iter()
        .any(|name| id == name),
        TokenTree::Group(group) => marked(group.stream()),
        _ => false,
    })
}

fn private_domain_arm(tokens: TokenStream) -> bool {
    let tokens: Vec<_> = tokens.into_iter().collect();
    tokens.windows(3).any(|tokens| {
        matches!(&tokens[0], TokenTree::Punct(value) if value.as_char() == '@')
            && matches!(&tokens[1], TokenTree::Ident(value) if value == "canonical")
            && matches!(&tokens[2], TokenTree::Ident(value) if value == "domain")
    }) || tokens
        .iter()
        .any(|token| matches!(token, TokenTree::Group(group) if private_domain_arm(group.stream())))
}

pub(super) fn discover(
    source: &str,
    text: &str,
    sources: &Sources<'_>,
) -> Result<Vec<Claim>, String> {
    // Only these platform implementation templates may generate claims.
    if [
        "workspaces/worth-query/crates/worth-query-declaration/src/value_binding_macro.rs",
        "workspaces/worth-query/crates/worth-query-declaration/src/binary_value_binding_macro.rs",
    ]
    .contains(&source)
    {
        return Ok(Vec::new());
    }
    let file = syn::parse_file(text).map_err(|error| format!("{source}: {error}"))?;
    let mut visitor = Discovery {
        source,
        sources,
        modules: Vec::new(),
        alias_scopes: Vec::new(),
        claims: Vec::new(),
        refusal: None,
        local_depth: 0,
    };
    visitor.visit_file(&file);
    match visitor.refusal {
        Some(error) => Err(error),
        None => Ok(visitor.claims),
    }
}

struct Discovery<'a> {
    source: &'a str,
    sources: &'a Sources<'a>,
    modules: Vec<String>,
    alias_scopes: Vec<BTreeSet<String>>,
    claims: Vec<Claim>,
    refusal: Option<String>,
    local_depth: usize,
}
impl Discovery<'_> {
    fn current_aliases(&self) -> BTreeSet<String> {
        self.alias_scopes.last().cloned().unwrap_or_default()
    }
    fn push(&mut self, binding: String, value: Type, origin: Option<syn::Path>) {
        if self.local_depth > 0 {
            self.refusal = Some(format!(
                "{}: domain bindings require an explicit module item, not a local scope",
                self.source
            ));
            return;
        }
        self.claims.push(Claim {
            source: self.source.to_owned(),
            modules: self.modules.clone(),
            binding,
            value,
            origin,
        });
    }
}
impl<'ast> Visit<'ast> for Discovery<'_> {
    fn visit_file(&mut self, file: &'ast syn::File) {
        let aliases = collect_binary_aliases(&file.items, &BTreeSet::new());
        self.alias_scopes.push(aliases);
        syn::visit::visit_file(self, file);
        self.alias_scopes.pop();
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.local_depth += 1;
        syn::visit::visit_block(self, block);
        self.local_depth -= 1;
    }
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        let parent_aliases = self.current_aliases();
        let aliases = module
            .content
            .as_ref()
            .map(|(_, items)| collect_binary_aliases(items, &parent_aliases))
            .unwrap_or_default();
        self.modules.push(module.ident.to_string());
        self.alias_scopes.push(aliases);
        syn::visit::visit_item_mod(self, module);
        self.alias_scopes.pop();
        self.modules.pop();
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if item.mac.path.is_ident("macro_rules") && marked(item.mac.tokens.clone()) {
            self.refusal = Some(format!("{}: domain bindings generated by an unknown macro require explicit source declarations", self.source));
            return;
        }
        syn::visit::visit_item_macro(self, item);
    }
    fn visit_macro(&mut self, mac: &'ast Macro) {
        if private_domain_arm(mac.tokens.clone()) {
            self.refusal = Some(format!("{}: internal domain declaration arms are unsupported; use the public domain_value declaration", self.source));
            return;
        }
        if let Ok(declaration) = syn::parse2::<Declaration>(mac.tokens.clone())
            && marked(declaration.body)
        {
            self.push(
                declaration.binding,
                declaration.value,
                Some(mac.path.clone()),
            );
        }
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let aliases = self.current_aliases();
        match inspect_worth_binary_derive(
            &item.attrs,
            &aliases,
            self.sources,
            self.source,
            &self.modules,
        ) {
            Ok(Some(origin)) => {
                if !item.generics.params.is_empty() {
                    self.refusal = Some(format!(
                        "{}: derived domain binary value `{}` must be a concrete non-generic struct",
                        self.source, item.ident
                    ));
                    return;
                }
                match derived_binary_binding_name(item) {
                    Ok(binding) => {
                        let value = syn::parse_str::<Type>(&item.ident.to_string())
                            .expect("struct ident is a valid type");
                        self.push(binding, value, Some(origin));
                    }
                    Err(error) => {
                        self.refusal = Some(format!("{}: {error}", self.source));
                        return;
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.refusal = Some(format!("{}: `{}`: {error}", self.source, item.ident));
                return;
            }
        }
        syn::visit::visit_item_struct(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let claim = canonical_binding(item) || item.items.iter().any(|member| match member {
            syn::ImplItem::Const(value) if value.ident == "DOMAIN_VALUE" =>
                !matches!(&value.expr, syn::Expr::Lit(value) if matches!(value.lit, syn::Lit::Bool(ref value) if !value.value)),
            _ => false,
        });
        if claim {
            let name = match item.self_ty.as_ref() {
                Type::Path(path) if path.qself.is_none() => path
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string()),
                _ => None,
            };
            let values: Vec<_> = item
                .items
                .iter()
                .filter_map(|member| match member {
                    syn::ImplItem::Type(value) if value.ident == "Value" => Some(value.ty.clone()),
                    _ => None,
                })
                .collect();
            match (name, values.as_slice(), item.generics.params.is_empty()) {
                (Some(name), [value], true) => self.push(name, value.clone(), None),
                _ => {
                    self.refusal = Some(format!(
                        "{}: domain binding needs a concrete associated Value",
                        self.source
                    ))
                }
            }
        }
        syn::visit::visit_item_impl(self, item);
    }
}
