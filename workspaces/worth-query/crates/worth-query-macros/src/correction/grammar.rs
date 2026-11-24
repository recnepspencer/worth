//! Attribute grammar for `#[derive(WorthQueryCorrection)]`.
use super::DERIVE;
use crate::grammar::{expr_list, unknown_key, value, visit_keys, Once};
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, LitStr, Path, Type};

const KEYS: &[&str] = &[
    "schema",
    "namespace",
    "identity",
    "source",
    "inverse",
    "disclosure",
    "source_body",
    "source_definition",
    "disclosure_parameters",
    "demand",
    "preimage_fields",
    "preimage_bytes",
    "derive",
    "inverse_key",
    "capability",
];

pub(super) struct CorrectionDeclaration {
    pub(super) schema: Type,
    pub(super) identity: CorrectionIdentity,
    pub(super) source: Type,
    pub(super) inverse: Type,
    pub(super) disclosure: Type,
    pub(super) source_body: Option<Path>,
    pub(super) source_definition: Option<Path>,
    pub(super) disclosure_parameters: Option<Path>,
    pub(super) demand: PreImageDemandSpec,
    pub(super) derive_fn: Path,
    pub(super) inverse_key: Option<Path>,
    pub(super) capability: Option<Type>,
}

pub(super) enum CorrectionIdentity {
    Namespace(Path),
    Explicit(LitStr),
}

pub(super) enum PreImageDemandSpec {
    Function(Path),
    Fields { fields: Vec<Expr>, bytes: Expr },
}

struct Keys {
    schema: Once<Type>,
    namespace: Once<Path>,
    identity: Once<LitStr>,
    source: Once<Type>,
    inverse: Once<Type>,
    disclosure: Once<Type>,
    source_body: Once<Path>,
    source_definition: Once<Path>,
    disclosure_parameters: Once<Path>,
    demand: Once<Path>,
    preimage_fields: Once<Vec<Expr>>,
    preimage_bytes: Once<Expr>,
    derive_fn: Once<Path>,
    inverse_key: Once<Path>,
    capability: Once<Type>,
}

impl CorrectionDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut keys = Keys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let identity = resolve_identity(keys.namespace, keys.identity, missing)?;
        let demand = resolve_demand(
            keys.demand,
            keys.preimage_fields,
            keys.preimage_bytes,
            missing,
        )?;
        Ok(Self {
            schema: keys.schema.required(missing, DERIVE)?,
            identity,
            source: keys.source.required(missing, DERIVE)?,
            inverse: keys.inverse.required(missing, DERIVE)?,
            disclosure: keys.disclosure.required(missing, DERIVE)?,
            source_body: keys.source_body.optional(),
            source_definition: keys.source_definition.optional(),
            disclosure_parameters: keys.disclosure_parameters.optional(),
            demand,
            derive_fn: keys.derive_fn.required(missing, DERIVE)?,
            inverse_key: keys.inverse_key.optional(),
            capability: keys.capability.optional(),
        })
    }
}

impl Keys {
    fn new() -> Self {
        Self {
            schema: Once::new("schema"),
            namespace: Once::new("namespace"),
            identity: Once::new("identity"),
            source: Once::new("source"),
            inverse: Once::new("inverse"),
            disclosure: Once::new("disclosure"),
            source_body: Once::new("source_body"),
            source_definition: Once::new("source_definition"),
            disclosure_parameters: Once::new("disclosure_parameters"),
            demand: Once::new("demand"),
            preimage_fields: Once::new("preimage_fields"),
            preimage_bytes: Once::new("preimage_bytes"),
            derive_fn: Once::new("derive"),
            inverse_key: Once::new("inverse_key"),
            capability: Once::new("capability"),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "schema" => self.schema.set(meta, value(meta)?),
            "namespace" => self.namespace.set(meta, value(meta)?),
            "identity" => self.identity.set(meta, value(meta)?),
            "source" => self.source.set(meta, value(meta)?),
            "inverse" => self.inverse.set(meta, value(meta)?),
            "disclosure" => self.disclosure.set(meta, value(meta)?),
            "source_body" => self.source_body.set(meta, value(meta)?),
            "source_definition" => self.source_definition.set(meta, value(meta)?),
            "disclosure_parameters" => self.disclosure_parameters.set(meta, value(meta)?),
            "demand" => self.demand.set(meta, value(meta)?),
            "preimage_fields" => self.preimage_fields.set(meta, expr_list(meta)?),
            "preimage_bytes" => self.preimage_bytes.set(meta, value(meta)?),
            "derive" => self.derive_fn.set(meta, value(meta)?),
            "inverse_key" => self.inverse_key.set(meta, value(meta)?),
            "capability" => self.capability.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, KEYS)),
        }
    }
}

fn resolve_identity(
    namespace: Once<Path>,
    identity: Once<LitStr>,
    missing: proc_macro2::Span,
) -> syn::Result<CorrectionIdentity> {
    if let (Some(_), Some(span)) = (namespace.span(), identity.span()) {
        return Err(syn::Error::new(
            span,
            "specify either `namespace = ..` or `identity = \"..\"`, not both",
        ));
    }
    if let Some(ns) = namespace.optional() {
        return Ok(CorrectionIdentity::Namespace(ns));
    }
    if let Some(id) = identity.optional() {
        return Ok(CorrectionIdentity::Explicit(id));
    }
    Err(syn::Error::new(
        missing,
        "#[derive(WorthQueryCorrection)] requires `namespace = NAMESPACE` or `identity = \"..\"`",
    ))
}

fn resolve_demand(
    demand: Once<Path>,
    fields: Once<Vec<Expr>>,
    bytes: Once<Expr>,
    missing: proc_macro2::Span,
) -> syn::Result<PreImageDemandSpec> {
    let inline_span = fields.span().or(bytes.span());
    if let Some(demand_fn) = demand.optional() {
        if let Some(span) = inline_span {
            return Err(syn::Error::new(
                span,
                "specify either `demand = fn` or `preimage_fields(..)` + `preimage_bytes = ..`, not both",
            ));
        }
        return Ok(PreImageDemandSpec::Function(demand_fn));
    }
    let Some(span) = inline_span else {
        return Err(syn::Error::new(
            missing,
            "#[derive(WorthQueryCorrection)] requires `demand = fn` or `preimage_fields(..), preimage_bytes = ..`",
        ));
    };
    let field_span = fields.span().unwrap_or(span);
    let fields = fields.required(span, DERIVE)?;
    if fields.is_empty() {
        return Err(syn::Error::new(
            field_span,
            "`preimage_fields(..)` must list at least one field selection",
        ));
    }
    let bytes = bytes.required(span, DERIVE)?;
    Ok(PreImageDemandSpec::Fields { fields, bytes })
}
