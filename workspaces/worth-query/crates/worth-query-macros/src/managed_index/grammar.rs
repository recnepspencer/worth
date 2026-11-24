//! Attribute parsing and validation for `#[derive(WorthManagedIndex)]`.
use super::DERIVE;
use crate::case::snake_case;
use crate::grammar::{member_name, unit_struct, unknown_key, value, visit_keys, Once};
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, Ident, LitInt, LitStr, Type};

const STRUCT_KEYS: &[&str] = &[
    "schema",
    "entity",
    "source",
    "probe",
    "probe_binding",
    "name",
    "provider",
    "provider_version",
    "source_scope",
    "structure",
    "membership",
    "maintenance",
    "refinement",
];

pub(super) struct ManagedIndexDeclaration {
    pub(super) schema: Type,
    pub(super) entity: Type,
    pub(super) source: Type,
    pub(super) probe: ProbeSpec,
    pub(super) name: LitStr,
    pub(super) provider: LitStr,
    pub(super) provider_version: u32,
    pub(super) source_scope: Option<Expr>,
    pub(super) structure: Option<Expr>,
    pub(super) membership: Option<Expr>,
    pub(super) maintenance: Option<Expr>,
    pub(super) refinement: Option<Expr>,
}

pub(super) enum ProbeSpec {
    ValueType(Type),
    BindingType(Type),
}

struct StructKeys {
    schema: Once<Type>,
    entity: Once<Type>,
    source: Once<Type>,
    probe: Once<Type>,
    probe_binding: Once<Type>,
    name: Once<LitStr>,
    provider: Once<LitStr>,
    provider_version: Once<u32>,
    source_scope: Once<Expr>,
    structure: Once<Expr>,
    membership: Once<Expr>,
    maintenance: Once<Expr>,
    refinement: Once<Expr>,
}

impl ManagedIndexDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        unit_struct(input, DERIVE)?;
        let mut keys = StructKeys {
            schema: Once::new("schema"),
            entity: Once::new("entity"),
            source: Once::new("source"),
            probe: Once::new("probe"),
            probe_binding: Once::new("probe_binding"),
            name: Once::new("name"),
            provider: Once::new("provider"),
            provider_version: Once::new("provider_version"),
            source_scope: Once::new("source_scope"),
            structure: Once::new("structure"),
            membership: Once::new("membership"),
            maintenance: Once::new("maintenance"),
            refinement: Once::new("refinement"),
        };
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let probe = match (keys.probe.span(), keys.probe_binding.span()) {
            (Some(_), Some(dup_span)) => {
                return Err(syn::Error::new(
                    dup_span,
                    "specify either `probe = ValueType` or `probe_binding = BindingType`, not both",
                ));
            }
            (Some(_), None) => ProbeSpec::ValueType(keys.probe.required(missing, DERIVE)?),
            (None, Some(_)) => {
                ProbeSpec::BindingType(keys.probe_binding.required(missing, DERIVE)?)
            }
            (None, None) => {
                return Err(syn::Error::new(
                    missing,
                    "#[derive(WorthManagedIndex)] requires `probe = ValueType` or `probe_binding = BindingType`",
                ));
            }
        };
        let provider_version = keys.provider_version.optional().unwrap_or(1);
        let name = keys.name.optional().unwrap_or_else(|| {
            LitStr::new(&snake_case(&member_name(&input.ident)), input.ident.span())
        });
        validate_token(&name, "name")?;
        let provider = keys.provider.optional().unwrap_or_else(|| {
            LitStr::new(
                &format!("worth.managed_index.{}.v{provider_version}", name.value()),
                name.span(),
            )
        });
        validate_token(&provider, "provider")?;
        Ok(Self {
            schema: keys.schema.required(missing, DERIVE)?,
            entity: keys.entity.required(missing, DERIVE)?,
            source: keys.source.required(missing, DERIVE)?,
            probe,
            name,
            provider,
            provider_version,
            source_scope: keys.source_scope.optional(),
            structure: keys.structure.optional(),
            membership: keys.membership.optional(),
            maintenance: keys.maintenance.optional(),
            refinement: keys.refinement.optional(),
        })
    }
}

impl StructKeys {
    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "schema" => self.schema.set(meta, value(meta)?),
            "entity" => self.entity.set(meta, value(meta)?),
            "source" => self.source.set(meta, value(meta)?),
            "probe" => self.probe.set(meta, value(meta)?),
            "probe_binding" => self.probe_binding.set(meta, value(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "provider" => self.provider.set(meta, value(meta)?),
            "provider_version" => {
                let lit: LitInt = value(meta)?;
                let parsed: u32 = lit.base10_parse().map_err(|_| {
                    syn::Error::new(lit.span(), "`provider_version` must be a u32 >= 1")
                })?;
                if parsed == 0 {
                    return Err(syn::Error::new(
                        lit.span(),
                        "`provider_version` must be at least 1",
                    ));
                }
                self.provider_version.set(meta, parsed)
            }
            "source_scope" => self.source_scope.set(meta, value(meta)?),
            "structure" => self.structure.set(meta, value(meta)?),
            "membership" => self.membership.set(meta, value(meta)?),
            "maintenance" => self.maintenance.set(meta, value(meta)?),
            "refinement" => self.refinement.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }
}

fn validate_token(lit: &LitStr, key: &str) -> syn::Result<()> {
    let text = lit.value();
    if text.is_empty() || text.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(syn::Error::new(
            lit.span(),
            format!("`{key}` must be non-empty and contain no whitespace"),
        ));
    }
    Ok(())
}

pub(super) fn bare_ident(expr: &Expr) -> Option<&Ident> {
    let Expr::Path(path) = expr else {
        return None;
    };
    if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 {
        let seg = &path.path.segments[0];
        if seg.arguments.is_empty() {
            return Some(&seg.ident);
        }
    }
    None
}
