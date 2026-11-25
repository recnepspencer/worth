//! Attribute parsing and validation for `#[derive(WorthManagedIndex)]`.
use super::DERIVE;
use crate::case::snake_case;
use crate::grammar::{member_name, unit_struct, unknown_key, value, visit_keys_for, Once};
use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Ident, LitInt, LitStr, Type};

const STRUCT_KEYS: &[&str] = &[
    "schema",
    "entity",
    "record",
    "source",
    "probe",
    "probe_binding",
    "name",
    "namespace",
    "provider",
    "version",
    "provider_version",
    "kind",
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
    pub(super) kind: Option<IndexProviderKind>,
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

pub(super) enum IndexProviderKind {
    TextPrefix {
        field: Box<Expr>,
        exact: bool,
    },
    Interval1D {
        start: Box<Expr>,
        end: Box<Expr>,
        closed: bool,
    },
    Box2D {
        min_x: Box<Expr>,
        min_y: Box<Expr>,
        max_x: Box<Expr>,
        max_y: Box<Expr>,
        closed: bool,
    },
    Box3D {
        min: Box<Expr>,
        max: Box<Expr>,
        closed: bool,
    },
    Custom {
        provider: Box<Expr>,
        watches: bool,
    },
}

impl IndexProviderKind {
    pub(super) fn exprs(&self) -> Vec<&Expr> {
        match self {
            Self::TextPrefix { field, .. } => vec![field],
            Self::Interval1D { start, end, .. } => vec![start, end],
            Self::Box2D {
                min_x,
                min_y,
                max_x,
                max_y,
                ..
            } => vec![min_x, min_y, max_x, max_y],
            Self::Box3D { min, max, .. } => vec![min, max],
            Self::Custom { provider, .. } => vec![provider],
        }
    }
}

struct StructKeys {
    schema: Once<Type>,
    entity: Once<Type>,
    record: Once<Type>,
    source: Once<Type>,
    probe: Once<Type>,
    probe_binding: Once<Type>,
    name: Once<LitStr>,
    namespace: Once<LitStr>,
    provider: Once<LitStr>,
    version: Once<u32>,
    provider_version: Once<u32>,
    kind: Once<IndexProviderKind>,
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
            record: Once::new("record"),
            source: Once::new("source"),
            probe: Once::new("probe"),
            probe_binding: Once::new("probe_binding"),
            name: Once::new("name"),
            namespace: Once::new("namespace"),
            provider: Once::new("provider"),
            version: Once::new("version"),
            provider_version: Once::new("provider_version"),
            kind: Once::new("kind"),
            source_scope: Once::new("source_scope"),
            structure: Once::new("structure"),
            membership: Once::new("membership"),
            maintenance: Once::new("maintenance"),
            refinement: Once::new("refinement"),
        };
        visit_keys_for(
            &input.attrs,
            &["worth_index", "worth_query"],
            |key, meta| keys.read(key, meta),
        )?;
        let missing = input.ident.span();
        let entity = match (keys.entity.span(), keys.record.span()) {
            (Some(_), Some(dup_span)) => {
                return Err(syn::Error::new(
                    dup_span,
                    "specify either `entity = Entity` or `record = Entity`, not both",
                ));
            }
            (Some(_), None) => keys.entity.required(missing, DERIVE)?,
            (None, Some(_)) => keys.record.required(missing, DERIVE)?,
            (None, None) => {
                return Err(syn::Error::new(
                    missing,
                    "#[derive(WorthManagedIndex)] requires `entity = Entity` (or `record = Entity`)",
                ));
            }
        };
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
        let provider_version = match (keys.provider_version.span(), keys.version.span()) {
            (Some(_), Some(dup_span)) => {
                return Err(syn::Error::new(
                    dup_span,
                    "specify either `version = N` or `provider_version = N`, not both",
                ));
            }
            (Some(_), None) => keys.provider_version.optional().unwrap_or(1),
            (None, Some(_)) => keys.version.optional().unwrap_or(1),
            (None, None) => 1,
        };
        let name = keys.name.optional().unwrap_or_else(|| {
            LitStr::new(&snake_case(&member_name(&input.ident)), input.ident.span())
        });
        validate_token(&name, "name")?;
        let provider = match (keys.provider.span(), keys.namespace.span()) {
            (Some(_), Some(dup_span)) => {
                return Err(syn::Error::new(
                    dup_span,
                    "specify either `provider = \"..\"` or `namespace = \"..\"`, not both",
                ));
            }
            (Some(_), None) => {
                let p = keys.provider.required(missing, DERIVE)?;
                validate_token(&p, "provider")?;
                p
            }
            (None, Some(_)) => {
                let ns = keys.namespace.required(missing, DERIVE)?;
                validate_token(&ns, "namespace")?;
                LitStr::new(&format!("{}.v{provider_version}", ns.value()), ns.span())
            }
            (None, None) => LitStr::new(
                &format!("worth.managed_index.{}.v{provider_version}", name.value()),
                name.span(),
            ),
        };
        Ok(Self {
            schema: keys.schema.required(missing, DERIVE)?,
            entity,
            source: keys.source.required(missing, DERIVE)?,
            probe,
            name,
            provider,
            provider_version,
            kind: keys.kind.optional(),
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
            "record" => self.record.set(meta, value(meta)?),
            "source" => self.source.set(meta, value(meta)?),
            "probe" => self.probe.set(meta, value(meta)?),
            "probe_binding" => self.probe_binding.set(meta, value(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "namespace" => self.namespace.set(meta, value(meta)?),
            "provider" => self.provider.set(meta, value(meta)?),
            "version" => self.version.set(meta, parse_u32_version(meta, "version")?),
            "provider_version" => self
                .provider_version
                .set(meta, parse_u32_version(meta, "provider_version")?),
            "kind" => self.kind.set(meta, parse_kind(meta)?),
            "source_scope" => self.source_scope.set(meta, value(meta)?),
            "structure" => self.structure.set(meta, value(meta)?),
            "membership" => self.membership.set(meta, value(meta)?),
            "maintenance" => self.maintenance.set(meta, value(meta)?),
            "refinement" => self.refinement.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }
}

fn parse_u32_version(meta: &ParseNestedMeta<'_>, key: &str) -> syn::Result<u32> {
    let lit: LitInt = value(meta)?;
    let parsed: u32 = lit
        .base10_parse()
        .map_err(|_| syn::Error::new(lit.span(), format!("`{key}` must be a u32 >= 1")))?;
    if parsed == 0 {
        return Err(syn::Error::new(
            lit.span(),
            format!("`{key}` must be at least 1"),
        ));
    }
    Ok(parsed)
}

fn parse_kind(meta: &ParseNestedMeta<'_>) -> syn::Result<IndexProviderKind> {
    let expr: Expr = value(meta)?;
    let span = expr.span();
    let Expr::Call(call) = expr else {
        return Err(syn::Error::new(
            span,
            "`kind` must be one of: text_prefix(field), text_prefix_exact(field), \
             interval_1d(start, end), interval_1d_closed(start, end), \
             box_2d(min_x, min_y, max_x, max_y), box_2d_closed(min_x, min_y, max_x, max_y), \
             box_3d((min_x, min_y, min_z), (max_x, max_y, max_z)), \
             box_3d_closed((min_x, min_y, min_z), (max_x, max_y, max_z)), \
             custom(provider), custom_with_watches(provider)",
        ));
    };
    let Some(name) = bare_ident(&call.func).map(|id| id.to_string()) else {
        return Err(syn::Error::new(
            call.func.span(),
            "expected a stock or custom index kind name",
        ));
    };
    let mut args: Vec<Expr> = call.args.into_iter().collect();
    match (name.as_str(), args.len()) {
        ("text_prefix", 1) => Ok(IndexProviderKind::TextPrefix {
            field: Box::new(args.remove(0)),
            exact: false,
        }),
        ("text_prefix_exact", 1) => Ok(IndexProviderKind::TextPrefix {
            field: Box::new(args.remove(0)),
            exact: true,
        }),
        ("interval_1d", 2) => Ok(IndexProviderKind::Interval1D {
            start: Box::new(args.remove(0)),
            end: Box::new(args.remove(0)),
            closed: false,
        }),
        ("interval_1d_closed", 2) => Ok(IndexProviderKind::Interval1D {
            start: Box::new(args.remove(0)),
            end: Box::new(args.remove(0)),
            closed: true,
        }),
        ("box_2d", 4) => Ok(IndexProviderKind::Box2D {
            min_x: Box::new(args.remove(0)),
            min_y: Box::new(args.remove(0)),
            max_x: Box::new(args.remove(0)),
            max_y: Box::new(args.remove(0)),
            closed: false,
        }),
        ("box_2d_closed", 4) => Ok(IndexProviderKind::Box2D {
            min_x: Box::new(args.remove(0)),
            min_y: Box::new(args.remove(0)),
            max_x: Box::new(args.remove(0)),
            max_y: Box::new(args.remove(0)),
            closed: true,
        }),
        ("box_3d", 2) => Ok(IndexProviderKind::Box3D {
            min: Box::new(args.remove(0)),
            max: Box::new(args.remove(0)),
            closed: false,
        }),
        ("box_3d_closed", 2) => Ok(IndexProviderKind::Box3D {
            min: Box::new(args.remove(0)),
            max: Box::new(args.remove(0)),
            closed: true,
        }),
        ("custom", 1) => Ok(IndexProviderKind::Custom {
            provider: Box::new(args.remove(0)),
            watches: false,
        }),
        ("custom_with_watches", 1) => Ok(IndexProviderKind::Custom {
            provider: Box::new(args.remove(0)),
            watches: true,
        }),
        _ => Err(syn::Error::new(
            span,
            format!("unsupported `kind = {name}(..)` or wrong argument count"),
        )),
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
