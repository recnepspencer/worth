use std::collections::BTreeSet;

use syn::{spanned::Spanned, Attribute, Data, DeriveInput, Fields, Ident, LitStr, Type};

pub(crate) struct Value {
    pub(crate) ident: Ident,
    pub(crate) wire_name: String,
    pub(crate) kind: ValueKind,
}

pub(crate) enum ValueKind {
    Struct(Vec<Field>),
    Newtype(Type),
    Unit,
    Enum(Vec<Variant>),
}

pub(crate) struct Field {
    pub(crate) ident: Ident,
    pub(crate) wire_name: String,
    pub(crate) ty: Type,
}

pub(crate) struct Variant {
    pub(crate) ident: Ident,
    pub(crate) wire_name: String,
    pub(crate) payload: Option<Type>,
}

impl Value {
    pub(crate) fn parse(input: DeriveInput) -> syn::Result<Self> {
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(syn::Error::new_spanned(
                input.generics,
                "WorthValue requires a concrete value type; generic values are unsupported",
            ));
        }
        let type_wire_name = wire_name(&input.attrs, &input.ident)?;
        let kind = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.into_iter().map(|field| {
                        let ident = field.ident.expect("named field has an identifier");
                        validate_type(&field.ty, &input.ident)?;
                        Ok(Field {
                            wire_name: wire_name(&field.attrs, &ident)?,
                            ident,
                            ty: field.ty,
                        })
                    }).collect::<syn::Result<Vec<_>>>()?;
                    require_distinct(fields.iter().map(|field| (&field.ident, &field.wire_name)))?;
                    ValueKind::Struct(fields)
                }
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let field = fields.unnamed.into_iter().next().expect("one newtype field");
                    reject_payload_attributes(&field.attrs)?;
                    validate_type(&field.ty, &input.ident)?;
                    ValueKind::Newtype(field.ty)
                }
                Fields::Unit => ValueKind::Unit,
                fields => return Err(syn::Error::new_spanned(
                    fields,
                    "WorthValue supports named or single-field newtype structs; tuple structs are unsupported",
                )),
            },
            Data::Enum(data) => {
                let variants = data.variants.into_iter().map(|variant| {
                    if variant.discriminant.is_some() {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "WorthValue does not support explicit enum discriminants",
                        ));
                    }
                    let payload = match variant.fields {
                        Fields::Unit => None,
                        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                            let field = fields.unnamed.into_iter().next().expect("one variant field");
                            reject_payload_attributes(&field.attrs)?;
                            validate_type(&field.ty, &input.ident)?;
                            Some(field.ty)
                        }
                        fields => return Err(syn::Error::new_spanned(
                            fields,
                            "WorthValue supports external unit or newtype enum variants; use a named payload struct",
                        )),
                    };
                    Ok(Variant {
                        wire_name: wire_name(&variant.attrs, &variant.ident)?,
                        ident: variant.ident,
                        payload,
                    })
                }).collect::<syn::Result<Vec<_>>>()?;
                if variants.is_empty() {
                    return Err(syn::Error::new(input.ident.span(), "WorthValue requires at least one enum variant"));
                }
                require_distinct(variants.iter().map(|variant| (&variant.ident, &variant.wire_name)))?;
                ValueKind::Enum(variants)
            }
            Data::Union(data) => return Err(syn::Error::new_spanned(
                data.union_token,
                "WorthValue does not support unions",
            )),
        };
        Ok(Self {
            ident: input.ident,
            wire_name: type_wire_name,
            kind,
        })
    }
}

fn wire_name(attributes: &[Attribute], ident: &Ident) -> syn::Result<String> {
    let mut rename = None;
    for attribute in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename") {
                return Err(meta.error(
                    "unsupported Serde attribute for WorthValue; only literal rename is supported; use an explicit bounded owner codec for custom wire behavior",
                ));
            }
            if rename.is_some() {
                return Err(meta.error("duplicate Serde rename"));
            }
            let literal: LitStr = meta.value()?.parse()?;
            rename = Some(literal.value());
            Ok(())
        })?;
    }
    Ok(rename.unwrap_or_else(|| ident.to_string().trim_start_matches("r#").to_owned()))
}

fn reject_payload_attributes(attributes: &[Attribute]) -> syn::Result<()> {
    if let Some(attribute) = attributes.iter().find(|attr| attr.path().is_ident("serde")) {
        return Err(syn::Error::new_spanned(
            attribute,
            "WorthValue newtype payloads do not support Serde attributes; place wire declarations on a named payload type",
        ));
    }
    Ok(())
}

fn require_distinct<'a>(names: impl Iterator<Item = (&'a Ident, &'a String)>) -> syn::Result<()> {
    let mut seen = BTreeSet::new();
    for (ident, name) in names {
        if !seen.insert(name) {
            return Err(syn::Error::new(
                ident.span(),
                format!("duplicate canonical wire name `{name}`"),
            ));
        }
    }
    Ok(())
}

fn validate_type(ty: &Type, owner: &Ident) -> syn::Result<()> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last().expect("type path has a segment");
            let name = segment.ident.to_string();
            if segment.ident == *owner || segment.ident == "Self" {
                return Err(syn::Error::new_spanned(ty, "WorthValue requires a finite nonrecursive wire shape; recursive values require an explicit bounded owner codec"));
            }
            match &segment.arguments {
                syn::PathArguments::None => {
                    if matches!(
                        name.as_str(),
                        "f32" | "f64" | "usize" | "isize" | "u128" | "i128"
                    ) {
                        return Err(syn::Error::new_spanned(ty, "unsupported canonical primitive; use a fixed-width supported integer or an explicit bounded owner codec"));
                    }
                    if path
                        .path
                        .segments
                        .iter()
                        .any(|part| !part.arguments.is_empty())
                    {
                        return Err(unsupported_type(ty));
                    }
                    Ok(())
                }
                syn::PathArguments::AngleBracketed(arguments)
                    if matches!(name.as_str(), "Vec" | "Option") && arguments.args.len() == 1 =>
                {
                    match arguments.args.first() {
                        Some(syn::GenericArgument::Type(child)) => validate_type(child, owner),
                        _ => Err(unsupported_type(ty)),
                    }
                }
                _ => Err(unsupported_type(ty)),
            }
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(()),
        _ => Err(unsupported_type(ty)),
    }
}

fn unsupported_type(ty: &Type) -> syn::Error {
    syn::Error::new(ty.span(), "unsupported WorthValue shape; use concrete derived values, Vec, or Option; arrays, tuples, and custom containers require an explicit bounded owner codec")
}
