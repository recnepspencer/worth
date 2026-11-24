//! Attribute and struct-shape validation for `#[derive(WorthBinaryValue)]`.
use super::DERIVE;
use crate::grammar::{no_generics, peeled, unknown_key, value, visit_keys, Once};
use quote::format_ident;
use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, LitInt, LitStr, Path, Type};

const STRUCT_KEYS: &[&str] = &[
    "identity", "version", "binding", "unit", "frame", "validate",
];

pub(super) struct BinaryValueDeclaration {
    pub(super) identity: LitStr,
    pub(super) version: u8,
    pub(super) binding: Ident,
    pub(super) unit: Option<LitStr>,
    pub(super) frame: Option<LitStr>,
    pub(super) validate: Option<Path>,
    pub(super) shape: StructShape,
}

pub(super) enum StructShape {
    Named(Vec<BinaryField>),
    Tuple(Vec<Type>),
    Unit,
}

pub(super) struct BinaryField {
    pub(super) ident: Ident,
    pub(super) ty: Type,
}

struct StructKeys {
    identity: Once<LitStr>,
    version: Once<u8>,
    binding: Once<Ident>,
    unit: Once<LitStr>,
    frame: Once<LitStr>,
    validate: Once<Path>,
}

impl BinaryValueDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        no_generics(input, DERIVE)?;
        let shape = parse_shape(input)?;
        let mut keys = StructKeys {
            identity: Once::new("identity"),
            version: Once::new("version"),
            binding: Once::new("binding"),
            unit: Once::new("unit"),
            frame: Once::new("frame"),
            validate: Once::new("validate"),
        };
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let identity = keys.identity.required(missing, DERIVE)?;
        validate_non_empty_token(&identity, "identity")?;
        let unit = keys.unit.optional();
        if let Some(ref u) = unit {
            validate_non_empty_token(u, "unit")?;
        }
        let frame = keys.frame.optional();
        if let Some(ref f) = frame {
            validate_non_empty_token(f, "frame")?;
        }
        let version = keys.version.optional().unwrap_or(1);
        let binding = keys
            .binding
            .optional()
            .unwrap_or_else(|| format_ident!("{}Binding", input.ident, span = input.ident.span()));
        Ok(Self {
            identity,
            version,
            binding,
            unit,
            frame,
            validate: keys.validate.optional(),
            shape,
        })
    }

    pub(super) fn field_types(&self) -> Vec<&Type> {
        match &self.shape {
            StructShape::Named(fields) => fields.iter().map(|f| &f.ty).collect(),
            StructShape::Tuple(types) => types.iter().collect(),
            StructShape::Unit => Vec::new(),
        }
    }
}

impl StructKeys {
    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "identity" => self.identity.set(meta, value(meta)?),
            "version" => {
                let lit: LitInt = value(meta)?;
                let parsed: u8 = lit.base10_parse().map_err(|_| {
                    syn::Error::new(
                        lit.span(),
                        "`version` must be an integer in 1..=255 for the wire version byte",
                    )
                })?;
                if parsed == 0 {
                    return Err(syn::Error::new(
                        lit.span(),
                        "`version` must be at least 1 (wire versions are 1..=255)",
                    ));
                }
                self.version.set(meta, parsed)
            }
            "binding" => self.binding.set(meta, value(meta)?),
            "unit" => self.unit.set(meta, value(meta)?),
            "frame" => self.frame.set(meta, value(meta)?),
            "validate" => self.validate.set(meta, validator(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }
}

fn parse_shape(input: &DeriveInput) -> syn::Result<StructShape> {
    let data = match &input.data {
        Data::Struct(data) => data,
        Data::Enum(data) => {
            return Err(syn::Error::new(
                data.enum_token.span,
                "#[derive(WorthBinaryValue)] applies only to structs; enums are not supported",
            ));
        }
        Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span,
                "#[derive(WorthBinaryValue)] applies only to structs; unions are not supported",
            ));
        }
    };
    match &data.fields {
        Fields::Named(named) => {
            let mut fields = Vec::with_capacity(named.named.len());
            for field in &named.named {
                reject_field_attrs(field)?;
                reject_raw_float(&field.ty)?;
                let ident = field.ident.clone().expect("named field has identifier");
                fields.push(BinaryField {
                    ident,
                    ty: field.ty.clone(),
                });
            }
            Ok(StructShape::Named(fields))
        }
        Fields::Unnamed(unnamed) => {
            let mut types = Vec::with_capacity(unnamed.unnamed.len());
            for field in &unnamed.unnamed {
                reject_field_attrs(field)?;
                reject_raw_float(&field.ty)?;
                types.push(field.ty.clone());
            }
            Ok(StructShape::Tuple(types))
        }
        Fields::Unit => Ok(StructShape::Unit),
    }
}

fn reject_field_attrs(field: &syn::Field) -> syn::Result<()> {
    if let Some(attr) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("worth_query"))
    {
        return Err(syn::Error::new(
            attr.span(),
            "#[derive(WorthBinaryValue)] does not accept field-level `#[worth_query(..)]` attributes",
        ));
    }
    Ok(())
}

fn reject_raw_float(ty: &Type) -> syn::Result<()> {
    let Type::Path(path) = peeled(ty) else {
        return Ok(());
    };
    if path.qself.is_none()
        && let Some(last) = path.path.segments.last()
        && last.arguments.is_empty()
        && (last.ident == "f32" || last.ident == "f64")
    {
        let canonical = if last.ident == "f32" {
            "CanonicalF32"
        } else {
            "CanonicalF64"
        };
        return Err(syn::Error::new(
            ty.span(),
            format!(
                "#[derive(WorthBinaryValue)] refuses raw `{}` because raw IEEE floats are non-canonical; use `{canonical}`",
                last.ident
            ),
        ));
    }
    Ok(())
}

fn validate_non_empty_token(lit: &LitStr, key: &str) -> syn::Result<()> {
    let text = lit.value();
    if text.is_empty() || text.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(syn::Error::new(
            lit.span(),
            format!("`{key}` must be non-empty and contain no whitespace"),
        ));
    }
    Ok(())
}

fn validator(meta: &ParseNestedMeta<'_>) -> syn::Result<Path> {
    value::<Path>(meta).map_err(|error| {
        syn::Error::new(
            error.span(),
            "`validate` takes a function path, such as `validate = rules::check_box`",
        )
    })
}
