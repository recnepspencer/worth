mod attributes;
pub(crate) mod types;
use attributes::Attributes;
use std::collections::BTreeSet;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Type};

pub(crate) struct Value {
    pub ident: Ident,
    pub facade: syn::Path,
    pub wire_name: String,
    pub kind: ValueKind,
}
pub(crate) enum ValueKind {
    Struct(Vec<Field>),
    Newtype(Type),
    Unit,
    Enum(Vec<Variant>),
    TaggedEnum {
        tag: String,
        variants: Vec<TaggedVariant>,
    },
}
#[derive(Clone)]
pub(crate) struct Field {
    pub ident: Ident,
    pub wire_name: String,
    pub ty: Type,
    pub delegate: Option<Type>,
}
pub(crate) struct Variant {
    pub ident: Ident,
    pub wire_name: String,
    pub payload: VariantPayload,
}
pub(crate) enum VariantPayload {
    Unit,
    Newtype(Type),
    Struct(Vec<Field>),
}
pub(crate) struct TaggedVariant {
    pub ident: Ident,
    pub wire_name: String,
    pub fields: Vec<Field>,
    pub unit: bool,
}

impl Value {
    pub(crate) fn parse(input: DeriveInput) -> syn::Result<Self> {
        let facade = attributes::facade_path(&input.attrs)?;
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(syn::Error::new_spanned(
                input.generics,
                "WorthValue requires a concrete value type; generic values are unsupported",
            ));
        }
        let attributes = Attributes::parse(&input.attrs, true)?;
        let wire_name = attributes.name(&input.ident, false);
        let kind = match input.data {
            Data::Struct(data) => {
                if attributes.tag.is_some() || attributes.snake_case {
                    return Err(syn::Error::new(input.ident.span(), "WorthValue tag/rename_all declarations belong on enums; use literal field renames"));
                }
                match data.fields {
                    Fields::Named(fields) => ValueKind::Struct(named_fields(fields)?),
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        let field = fields.unnamed.into_iter().next().expect("one newtype field");
                        reject_payload_attributes(&field.attrs)?;
                        types::delegate(&field.ty)?;
                        ValueKind::Newtype(field.ty)
                    },
                    Fields::Unit => ValueKind::Unit,
                    fields => return Err(syn::Error::new_spanned(fields, "WorthValue supports named or single-field newtype structs; tuple structs are unsupported")),
                }
            }
            Data::Enum(data) => {
                if data.variants.is_empty() {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "WorthValue requires at least one enum variant",
                    ));
                }
                if let Some(tag) = attributes.tag {
                    let variants = data.variants.into_iter().map(|variant| {
                        reject_delegate_attributes(&variant.attrs)?;
                        if variant.discriminant.is_some() { return Err(syn::Error::new_spanned(variant, "WorthValue does not support explicit enum discriminants")); }
                        let name = Attributes::parse(&variant.attrs, false)?.name(&variant.ident, attributes.snake_case);
                        let (fields, unit) = match variant.fields {
                            Fields::Named(fields) => (named_fields(fields)?, false),
                            Fields::Unit => (vec![], true),
                            fields => return Err(syn::Error::new_spanned(fields, "WorthValue internally tagged enums require unit or named variants; tuple/newtype variants are unsupported")),
                        };
                        if fields.iter().any(|field| field.wire_name == tag) { return Err(syn::Error::new(variant.ident.span(), "canonical field name conflicts with the enum tag")); }
                        Ok(TaggedVariant { ident: variant.ident, wire_name: name, fields, unit })
                    }).collect::<syn::Result<Vec<_>>>()?;
                    distinct(variants.iter().map(|v| (&v.ident, &v.wire_name)))?;
                    ValueKind::TaggedEnum { tag, variants }
                } else {
                    let variants = data.variants.into_iter().map(|variant| {
                        reject_delegate_attributes(&variant.attrs)?;
                        if variant.discriminant.is_some() { return Err(syn::Error::new_spanned(variant, "WorthValue does not support explicit enum discriminants")); }
                        let payload = match variant.fields {
                            Fields::Unit => VariantPayload::Unit,
                            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                                let field = fields.unnamed.into_iter().next().expect("one variant field");
                                reject_payload_attributes(&field.attrs)?;
                                types::delegate(&field.ty)?;
                                VariantPayload::Newtype(field.ty)
                            },
                            Fields::Named(fields) => VariantPayload::Struct(named_fields(fields)?),
                            fields => return Err(syn::Error::new_spanned(fields, "WorthValue supports external unit, newtype or named variants; tuple variants require a named payload struct")),
                        };
                        Ok(Variant { wire_name: Attributes::parse(&variant.attrs, false)?.name(&variant.ident, attributes.snake_case), ident: variant.ident, payload })
                    }).collect::<syn::Result<Vec<_>>>()?;
                    distinct(variants.iter().map(|v| (&v.ident, &v.wire_name)))?;
                    ValueKind::Enum(variants)
                }
            }
            Data::Union(data) => {
                return Err(syn::Error::new_spanned(
                    data.union_token,
                    "WorthValue does not support unions",
                ))
            }
        };
        Ok(Self {
            ident: input.ident,
            facade,
            wire_name,
            kind,
        })
    }
}
fn named_fields(fields: syn::FieldsNamed) -> syn::Result<Vec<Field>> {
    let fields = fields
        .named
        .into_iter()
        .map(|field| {
            let ident = field.ident.expect("named field");
            let delegate = field_delegate(&field.attrs)?;
            if delegate.is_none() {
                types::delegate(&field.ty)?;
            }
            Ok(Field {
                wire_name: Attributes::parse(&field.attrs, false)?.name(&ident, false),
                ident,
                ty: field.ty,
                delegate,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    distinct(fields.iter().map(|field| (&field.ident, &field.wire_name)))?;
    Ok(fields)
}
fn reject_payload_attributes(attributes: &[Attribute]) -> syn::Result<()> {
    reject_delegate_attributes(attributes)?;
    if let Some(attribute) = attributes.iter().find(|attr| attr.path().is_ident("serde")) {
        return Err(syn::Error::new_spanned(attribute, "WorthValue newtype payloads do not support Serde attributes; place wire declarations on a named payload type"));
    }
    Ok(())
}
fn field_delegate(attributes: &[Attribute]) -> syn::Result<Option<Type>> {
    let mut delegate = None;
    for attribute in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("worth_value"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("delegate") || delegate.is_some() {
                return Err(meta.error("WorthValue fields accept one explicit `delegate = Type`"));
            }
            delegate = Some(meta.value()?.parse::<Type>()?);
            Ok(())
        })?;
    }
    Ok(delegate)
}
fn reject_delegate_attributes(attributes: &[Attribute]) -> syn::Result<()> {
    if let Some(attribute) = attributes
        .iter()
        .find(|attr| attr.path().is_ident("worth_value"))
    {
        return Err(syn::Error::new_spanned(
            attribute,
            "WorthValue delegates belong on named fields",
        ));
    }
    Ok(())
}
fn distinct<'a>(names: impl Iterator<Item = (&'a Ident, &'a String)>) -> syn::Result<()> {
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
