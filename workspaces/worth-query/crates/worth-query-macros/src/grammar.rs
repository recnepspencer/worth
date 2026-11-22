//! The `#[worth_query(..)]` attribute grammar every derive shares: key
//! dispatch, once-only keys, value forms and the struct shapes a derive admits.
use proc_macro2::{Span, TokenStream};
use syn::ext::IdentExt;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Fields, FieldsNamed, GenericArgument, Ident, LitStr,
    PathArguments, Token, Type,
};

const ATTRIBUTE: &str = "worth_query";

/// Visits each key of every `#[worth_query(..)]` attribute, in source order.
pub(crate) fn visit_keys(
    attributes: &[Attribute],
    mut visit: impl FnMut(&str, &ParseNestedMeta<'_>) -> syn::Result<()>,
) -> syn::Result<()> {
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident(ATTRIBUTE))
    {
        attribute.parse_nested_meta(|meta| {
            let key = meta
                .path
                .get_ident()
                .ok_or_else(|| meta.error("expected a `worth_query` key, such as `schema = ..`"))?
                .unraw()
                .to_string();
            visit(&key, &meta)
        })?;
    }
    Ok(())
}

/// A key that may appear at most once, remembering where it was written.
pub(crate) struct Once<T> {
    key: &'static str,
    value: Option<(T, Span)>,
}

impl<T> Once<T> {
    pub(crate) const fn new(key: &'static str) -> Self {
        Self { key, value: None }
    }

    pub(crate) fn set(&mut self, meta: &ParseNestedMeta<'_>, value: T) -> syn::Result<()> {
        if self.value.is_some() {
            return Err(meta.error(format!("duplicate `{}` key", self.key)));
        }
        self.value = Some((value, meta.path.span()));
        Ok(())
    }

    pub(crate) fn is_set(&self) -> bool {
        self.value.is_some()
    }

    /// Where the key was written, if it was.
    pub(crate) fn span(&self) -> Option<Span> {
        self.value.as_ref().map(|(_, span)| *span)
    }

    pub(crate) fn optional(self) -> Option<T> {
        self.value.map(|(value, _)| value)
    }

    /// The value, or an error at `missing` naming the key to add.
    pub(crate) fn required(self, missing: Span, derive: &str) -> syn::Result<T> {
        let key = self.key;
        self.optional().ok_or_else(|| {
            syn::Error::new(
                missing,
                format!("#[derive({derive})] requires `#[worth_query({key} = ..)]`"),
            )
        })
    }

    /// Refuses this key when the declaration does not use it.
    pub(crate) fn refuse_unless(&self, allowed: bool, role: &str) -> syn::Result<()> {
        match self.span() {
            Some(span) if !allowed => Err(syn::Error::new(
                span,
                format!("`{}` does not apply to {role}", self.key),
            )),
            _ => Ok(()),
        }
    }
}

pub(crate) fn value<T: syn::parse::Parse>(meta: &ParseNestedMeta<'_>) -> syn::Result<T> {
    meta.value()?.parse()
}

/// `key(A, B, ..)`: a parenthesized, comma-separated list of types.
pub(crate) fn type_list(meta: &ParseNestedMeta<'_>) -> syn::Result<Vec<Type>> {
    let content;
    syn::parenthesized!(content in meta.input);
    Ok(Punctuated::<Type, Token![,]>::parse_terminated(&content)?
        .into_iter()
        .collect())
}

/// `key(..)`: parenthesized tokens forwarded verbatim to a declaration macro.
pub(crate) fn token_group(meta: &ParseNestedMeta<'_>) -> syn::Result<TokenStream> {
    let content;
    syn::parenthesized!(content in meta.input);
    content.parse()
}

/// A key written alone, without `= value` or `(..)`.
pub(crate) fn flag(meta: &ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.input.is_empty() || meta.input.peek(Token![,]) {
        Ok(true)
    } else {
        Err(meta.error("this key is a flag and takes no value"))
    }
}

pub(crate) fn unknown_key(meta: &ParseNestedMeta<'_>, derive: &str, keys: &[&str]) -> syn::Error {
    meta.error(format!(
        "unknown key for #[derive({derive})]; expected one of: {}",
        keys.join(", ")
    ))
}

/// The struct's named fields; tuple and unit structs, enums and unions are
/// refused at their defining token.
pub(crate) fn named_fields<'a>(
    input: &'a DeriveInput,
    derive: &str,
) -> syn::Result<&'a FieldsNamed> {
    no_generics(input, derive)?;
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(syn::Error::new(
                input.ident.span(),
                format!("#[derive({derive})] requires a struct with named fields"),
            )),
        },
        Data::Enum(data) => Err(not_a_struct(data.enum_token.span, derive)),
        Data::Union(data) => Err(not_a_struct(data.union_token.span, derive)),
    }
}

/// Refuses anything but a unit struct marker.
pub(crate) fn unit_struct(input: &DeriveInput, derive: &str) -> syn::Result<()> {
    no_generics(input, derive)?;
    match &input.data {
        Data::Struct(data) if matches!(data.fields, Fields::Unit) => Ok(()),
        Data::Struct(data) => Err(syn::Error::new(
            data.fields.span(),
            format!("#[derive({derive})] declares a marker; use a unit struct"),
        )),
        Data::Enum(data) => Err(not_a_struct(data.enum_token.span, derive)),
        Data::Union(data) => Err(not_a_struct(data.union_token.span, derive)),
    }
}

fn no_generics(input: &DeriveInput, derive: &str) -> syn::Result<()> {
    match input.generics.params.first() {
        Some(parameter) => Err(syn::Error::new(
            parameter.span(),
            format!("#[derive({derive})] does not support generic parameters"),
        )),
        None => Ok(()),
    }
}

fn not_a_struct(span: Span, derive: &str) -> syn::Error {
    syn::Error::new(span, format!("#[derive({derive})] applies only to structs"))
}

/// `T` when `ty` is spelled `Option<T>` (any path ending in `Option`).
pub(crate) fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let last = path.path.segments.last()?;
    let PathArguments::AngleBracketed(arguments) = &last.arguments else {
        return None;
    };
    match (
        path.qself.is_none() && last.ident == "Option",
        arguments.args.first(),
    ) {
        (true, Some(GenericArgument::Type(inner))) if arguments.args.len() == 1 => Some(inner),
        _ => None,
    }
}

/// The column a `FieldSelection` alias selects: an explicit `column`, else
/// the alias's own name, as record selection aliases are named.
pub(crate) fn selection_column(
    selection: &Type,
    column: Option<LitStr>,
    missing: Span,
) -> syn::Result<LitStr> {
    let alias = match selection {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last(),
        _ => None,
    };
    match (column, alias) {
        (Some(column), _) => Ok(column),
        (None, Some(alias)) => Ok(LitStr::new(&alias.ident.to_string(), alias.ident.span())),
        (None, None) => Err(syn::Error::new(
            missing,
            "this selection is not a named type; add `column = \"..\"`",
        )),
    }
}

/// The field's identifier without a raw prefix.
pub(crate) fn member_name(member: &Ident) -> String {
    member.unraw().to_string()
}
