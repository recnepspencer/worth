//! `#[derive(WorthQueryAbilities)]`: one ability marker per unit variant, all
//! scoped to one entity, and the vocabulary `role_abilities` installs.
//!
//! Enum keys: `schema = Schema` and `scoped_to = Scope` (both required) and
//! `module = name` (default the enum's snake-case name). Each variant `V`
//! becomes `module::V`, declared by `worth_query_ability!` exactly as a
//! hand-written marker, so its installed identifier is the variant name.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Fields, Ident, Type};

use crate::facade_paths::declaration;
use crate::grammar::{enum_variants, unknown_key, value, visit_keys, Once};

pub(crate) const DERIVE: &str = "WorthQueryAbilities";
const KEYS: &[&str] = &["schema", "scoped_to", "module"];

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let mut schema = Once::<Type>::new("schema");
    let mut scope = Once::<Type>::new("scoped_to");
    let mut module = Once::<Ident>::new("module");
    visit_keys(&input.attrs, |key, meta| match key {
        "schema" => schema.set(meta, value(meta)?),
        "scoped_to" => scope.set(meta, value(meta)?),
        "module" => module.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, KEYS)),
    })?;
    let missing = input.ident.span();
    let schema = schema.required(missing, DERIVE)?;
    let scope = scope.required(missing, DERIVE)?;
    let module = module
        .optional()
        .unwrap_or_else(|| format_ident!("{}", snake_case(&input.ident.to_string())));
    let mut variants = Vec::new();
    for variant in enum_variants(input, DERIVE, "an ability")? {
        if !matches!(variant.fields, Fields::Unit) || variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "an ability is a unit variant without a code; its name is its identifier",
            ));
        }
        variants.push(variant.ident.clone());
    }
    let (decl, vis, abilities) = (declaration(), &input.vis, &input.ident);
    let first = variants.first().ok_or_else(|| {
        syn::Error::new(
            missing,
            format!("#[derive({DERIVE})] needs at least one ability"),
        )
    })?;
    Ok(quote! {
        /// One ability marker per variant, as `worth_query_ability!` declares it.
        #vis mod #module {
            #[allow(unused_imports)]
            use super::*;
            #(#decl::worth_query_ability!(pub #variants scoped_to #scope, in #schema);)*
        }

        impl #decl::application_schema::ApplicationAbilityVocabulary<#schema> for #abilities {
            type Scope = #scope;

            const ALL: &'static [Self] = &[#(Self::#variants),*];

            fn ability_name(self) -> &'static str {
                match self {
                    #(Self::#variants => #module::#variants::reference().name(),)*
                }
            }

            fn scope_name() -> &'static str {
                #module::#first::reference().scope()
            }
        }
    })
}

fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() {
            if index > 0 {
                snake.push('_');
            }
            snake.extend(character.to_lowercase());
        } else {
            snake.push(character);
        }
    }
    snake
}
