//! Discriminator keys, and the variants and fields whose names are persisted.
use super::DERIVE;
use crate::grammar::{enum_variants, no_generics, unknown_key, value, visit_keys, wire_name, Once};
use std::collections::BTreeSet;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Token, Type};

const TYPE_KEYS: &[&str] = &["identity", "referenced_roles"];
const MEMBER_KEYS: &[&str] = &["name"];

pub(super) struct DiscriminatorDeclaration {
    pub(super) identity: LitStr,
    /// `None` keeps the trait's default of no referenced role.
    pub(super) referenced_roles: Option<Vec<LitStr>>,
    pub(super) shape: Shape,
}

pub(super) enum Shape {
    Struct(Members),
    Enum(Vec<VariantDeclaration>),
}

pub(super) struct VariantDeclaration {
    pub(super) variant: Ident,
    pub(super) name: LitStr,
    pub(super) members: Members,
}

pub(super) enum Members {
    Unit,
    /// Fields written under their persisted names.
    Named(Vec<(Ident, LitStr, Type)>),
    /// Fields written by position.
    Positional(Vec<Type>),
}

impl DiscriminatorDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut identity = Once::<LitStr>::new("identity");
        let mut referenced_roles = Once::<Vec<LitStr>>::new("referenced_roles");
        visit_keys(&input.attrs, |key, meta| match key {
            "identity" => identity.set(meta, value(meta)?),
            "referenced_roles" => referenced_roles.set(meta, string_list(meta)?),
            _ => Err(unknown_key(meta, DERIVE, TYPE_KEYS)),
        })?;
        let identity = identity.required(input.ident.span(), DERIVE)?;
        refuse_malformed_identity(&identity)?;
        let shape = match &input.data {
            Data::Struct(data) => {
                no_generics(input, DERIVE)?;
                Shape::Struct(members(&data.fields)?)
            }
            Data::Enum(_) => Shape::Enum(variants(input)?),
            Data::Union(data) => {
                return Err(syn::Error::new(
                    data.union_token.span,
                    format!("#[derive({DERIVE})] applies only to structs and enums"),
                ));
            }
        };
        Ok(Self {
            identity,
            referenced_roles: referenced_roles.optional(),
            shape,
        })
    }
}

/// `key = ["a", "b", ..]`: a bracketed, comma-separated list of strings.
fn string_list(meta: &ParseNestedMeta<'_>) -> syn::Result<Vec<LitStr>> {
    let list = meta.value()?;
    let content;
    syn::bracketed!(content in list);
    let roles = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
    let mut written = BTreeSet::new();
    for role in &roles {
        if !written.insert(role.value()) {
            return Err(syn::Error::new(
                role.span(),
                format!("duplicate referenced role `{}`", role.value()),
            ));
        }
    }
    Ok(roles.into_iter().collect())
}

/// The identity grammar installation checks: dotted, not blank, no whitespace.
fn refuse_malformed_identity(identity: &LitStr) -> syn::Result<()> {
    let text = identity.value();
    let malformed = text.is_empty()
        || text.starts_with('.')
        || text.ends_with('.')
        || text.contains("..")
        || text.chars().any(|ch| ch.is_whitespace() || ch.is_control());
    match malformed {
        true => Err(syn::Error::new(
            identity.span(),
            "a discriminator identity is a dotted name such as `app.face.cut.v1`: not empty, \
             no whitespace, no empty segment",
        )),
        false => Ok(()),
    }
}

fn variants(input: &DeriveInput) -> syn::Result<Vec<VariantDeclaration>> {
    let mut written = BTreeSet::new();
    enum_variants(input, DERIVE, "one case of the discriminator")?
        .iter()
        .map(|variant| {
            let name = wire_name(name_key(&variant.attrs)?, &variant.ident, "variant")?;
            if !written.insert(name.value()) {
                return Err(syn::Error::new(
                    name.span(),
                    format!("duplicate variant name `{}`", name.value()),
                ));
            }
            Ok(VariantDeclaration {
                variant: variant.ident.clone(),
                name,
                members: members(&variant.fields)?,
            })
        })
        .collect()
}

fn members(fields: &Fields) -> syn::Result<Members> {
    match fields {
        Fields::Unit => Ok(Members::Unit),
        Fields::Named(fields) => {
            let mut written = BTreeSet::new();
            let named = fields.named.iter().map(|field| {
                let member = field.ident.clone().expect("named fields carry identifiers");
                let name = wire_name(name_key(&field.attrs)?, &member, "field")?;
                if !written.insert(name.value()) {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("duplicate field name `{}`", name.value()),
                    ));
                }
                Ok((member, name, field.ty.clone()))
            });
            Ok(Members::Named(named.collect::<syn::Result<_>>()?))
        }
        Fields::Unnamed(fields) => {
            let positional = fields
                .unnamed
                .iter()
                .map(|field| match name_key(&field.attrs)? {
                    Some(name) => Err(syn::Error::new(
                        name.span(),
                        "a tuple field is written by position and has no name",
                    )),
                    None => Ok(field.ty.clone()),
                });
            Ok(Members::Positional(positional.collect::<syn::Result<_>>()?))
        }
    }
}

/// The explicit `#[worth_query(name = "..")]` of a variant or field.
fn name_key(attributes: &[Attribute]) -> syn::Result<Option<LitStr>> {
    let mut name = Once::<LitStr>::new("name");
    visit_keys(attributes, |key, meta| match key {
        "name" => name.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, MEMBER_KEYS)),
    })?;
    Ok(name.optional())
}

impl Members {
    /// How many fields are written.
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Unit => 0,
            Self::Named(fields) => fields.len(),
            Self::Positional(types) => types.len(),
        }
    }
}
