//! Effect-driven live mode for `#[derive(WorthQuery)]`:
//! `live = effect(Effect, row_key = Row::key(), payload_key = path_or_closure)`.
//!
//! The query's single root is its scope. A committed `Effect` invalidates
//! the live result of the root whose `row_key` equals `payload_key(payload)`.
//! The derive emits the `<Request>LiveCause` marker with its
//! `ApplicationQueryLiveCauseBinding`, the request's `ApplicationLiveQueryIntent`,
//! and the definition's `live_root_by` step, so no cause or intent is
//! written by hand.
use super::grammar::QueryDeclaration;
use crate::facade_paths::{declaration, fragment};
use proc_macro2::TokenTree;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::{DeriveInput, Ident, Token, Type};

const KEYS: [&str; 2] = ["row_key", "payload_key"];

/// Values are kept as tokens: a `payload_key` closure parses without syn's
/// full expression grammar and type-checks against the generated signature.
pub(in crate::query) struct LiveEffect {
    pub(in crate::query) effect: Type,
    pub(in crate::query) row_key: TokenStream,
    pub(in crate::query) payload_key: TokenStream,
}

impl LiveEffect {
    /// Parses the parenthesized body of `effect(..)`.
    pub(in crate::query) fn parse(content: ParseStream<'_>) -> syn::Result<Self> {
        let effect: Type = content.parse()?;
        let (mut row_key, mut payload_key) = (None, None);
        while !content.is_empty() {
            content.parse::<Token![,]>()?;
            if content.is_empty() {
                break;
            }
            let key: Ident = content.parse()?;
            content.parse::<Token![=]>()?;
            let slot = match key.to_string().as_str() {
                "row_key" => &mut row_key,
                "payload_key" => &mut payload_key,
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                        "unknown `effect(..)` key `{other}`; expected `row_key` or `payload_key`"
                    ),
                    ))
                }
            };
            let value = value_tokens(content)?;
            if value.is_empty() {
                return Err(syn::Error::new(
                    key.span(),
                    format!("`{key}` needs a value"),
                ));
            }
            if slot.replace(value).is_some() {
                return Err(syn::Error::new(
                    key.span(),
                    format!("`{key}` is written twice"),
                ));
            }
        }
        let missing = |name: &str| {
            content.error(format!(
                "`live = effect(Effect, row_key = Row::key(), payload_key = ..)` requires `{name}`"
            ))
        };
        Ok(Self {
            effect,
            row_key: row_key.ok_or_else(|| missing("row_key"))?,
            payload_key: payload_key.ok_or_else(|| missing("payload_key"))?,
        })
    }
}

/// One key's value: every token up to the next `, key =` or the end.
fn value_tokens(content: ParseStream<'_>) -> syn::Result<TokenStream> {
    let mut value = TokenStream::new();
    while !content.is_empty() && !(content.peek(Token![,]) && next_is_key(content)) {
        value.extend([content.parse::<TokenTree>()?]);
    }
    Ok(value)
}

fn next_is_key(content: ParseStream<'_>) -> bool {
    let fork = content.fork();
    let _ = fork.parse::<Token![,]>();
    fork.is_empty()
        || (fork
            .parse::<Ident>()
            .is_ok_and(|key| KEYS.contains(&key.to_string().as_str()))
            && fork.peek(Token![=])
            && !fork.peek(Token![==]))
}

/// The generated live cause marker: `<Request>LiveCause`.
pub(in crate::query) fn cause_marker(input: &DeriveInput) -> Ident {
    format_ident!("{}LiveCause", input.ident, span = input.ident.span())
}

/// The `live_root_by` definition step over the row key.
pub(in crate::query) fn live_step(input: &DeriveInput, live: &LiveEffect) -> TokenStream {
    let (cause, row_key) = (cause_marker(input), &live.row_key);
    quote!(.live_root_by::<#cause>(#row_key, #row_key))
}

/// The cause marker, its portable identity, its cause binding over the
/// scope key, and the request's live intent.
pub(in crate::query) fn cause_items(
    input: &DeriveInput,
    decl_spec: &QueryDeclaration,
    live: &LiveEffect,
) -> TokenStream {
    let (decl, frag, vis, request) = (declaration(), fragment(), &input.vis, &input.ident);
    let (schema, query, namespace) = (&decl_spec.schema, &decl_spec.query, &decl_spec.namespace);
    let (effect, payload_key) = (&live.effect, &live.payload_key);
    let (selection, _) = decl_spec.scope.selected_column();
    let cause = cause_marker(input);
    let doc = format!("Live cause of `{request}`: a committed effect invalidates one root.");
    let (schema_alias, payload, key) = (
        Ident::new("WorthLiveSchema", Span::mixed_site()),
        Ident::new("payload", Span::mixed_site()),
        Ident::new("key", Span::mixed_site()),
    );
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let scope_entity = quote!(<#selection as #frag::RecordSelection>::Entity);
    let key_binding = quote!(<#selection as #frag::RecordSelection>::Binding);
    let key_value = quote!(<#key_binding as #aschema::ApplicationScalarValueBinding>::Value);
    let payload_binding = quote!(<#effect as #aschema::ApplicationEffectMarkerIdentity<#schema_alias>>::PayloadBinding);
    let payload_value =
        quote!(<#payload_binding as #aschema::ApplicationStructuredValueBinding>::Value);
    quote! {
        #[doc = #doc]
        #vis struct #cause;
        #decl::worth_query_portable_type!(#cause => #decl::worth_query_identity!(#namespace, live_cause));
        const _: () = {
            type #schema_alias = #schema;
            impl #aq::ApplicationQueryLiveCauseBinding<#schema_alias, #query, #scope_entity, #scope_entity>
                for #cause
            {
                type Effect = #effect;
                type PayloadBinding = #payload_binding;
                type ScopeIdentityBinding = #key_binding;
                type TargetIdentityBinding = #key_binding;
                fn effect() -> #aschema::ApplicationEffectRef<#schema_alias, #effect, #payload_value> {
                    #aschema::ApplicationEffectRef::from_declaration()
                }
                fn scope_identity(#payload: &#payload_value) -> #key_value {
                    let #key: fn(&#payload_value) -> #key_value = #payload_key;
                    #key(#payload)
                }
                fn target_identity(#payload: &#payload_value) -> #key_value {
                    <Self as #aq::ApplicationQueryLiveCauseBinding<
                        #schema_alias, #query, #scope_entity, #scope_entity,
                    >>::scope_identity(#payload)
                }
            }
            impl #aq::ApplicationLiveQueryIntent<#schema_alias> for #request {
                type Target = #scope_entity;
                type LiveCause = #cause;
            }
        };
    }
}
