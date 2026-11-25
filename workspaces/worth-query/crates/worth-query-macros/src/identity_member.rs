//! The `namespace = NS` key with an optional `member`: several derives share
//! one identity namespace, each under its own member segment.
//!
//! `member` alone takes the snake_case Rust item name; `member = ".."` names
//! the segment explicitly. Identities then derive as `root.<member>.<role>`
//! through `worth_query_identity_member!`, whose const evaluation refuses a
//! reserved or malformed segment at compile time.
use crate::case::snake_case;
use crate::facade_paths::declaration;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::meta::ParseNestedMeta;
use syn::{Ident, LitStr, Path, Token};

/// The raw `member` key: `None` derives the segment from the item name.
pub(crate) type RawMember = Option<LitStr>;

pub(crate) fn parse_member(meta: &ParseNestedMeta<'_>) -> syn::Result<RawMember> {
    if meta.input.peek(Token![=]) {
        meta.value()?;
        return Ok(Some(meta.input.parse()?));
    }
    Ok(None)
}

/// A derive's identity namespace: the declared path, optionally narrowed to
/// one member.
#[derive(Clone)]
pub(crate) struct DeclaredNamespace {
    path: Path,
    member: Option<LitStr>,
}

impl DeclaredNamespace {
    pub(crate) fn new(path: Path, member: Option<RawMember>, item: &Ident) -> Self {
        let member = member.map(|explicit| {
            explicit.unwrap_or_else(|| LitStr::new(&snake_case(&item.to_string()), item.span()))
        });
        Self { path, member }
    }
}

impl ToTokens for DeclaredNamespace {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.path;
        match &self.member {
            Some(member) => {
                let decl = declaration();
                tokens.extend(quote!(#decl::worth_query_identity_member!(#path, #member)));
            }
            None => path.to_tokens(tokens),
        }
    }
}
