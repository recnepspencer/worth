//! Attribute grammar for `#[derive(WorthQueryCorrection)]`.
use super::DERIVE;
use crate::grammar::{expr_list, flag, unknown_key, value, visit_keys, Once};
use syn::ext::IdentExt;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, Ident, LitStr, Path, Type};

const KEYS: &[&str] = &[
    "mode",
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
    "reapply",
    "reapplication_key",
    "program",
    "inverse_action",
    "source_action",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CorrectionMode {
    RecordedInverse,
    AuthoredBody,
}

pub(super) struct AuthoredBodyActions {
    pub(super) program: Type,
    pub(super) inverse_action: Type,
    pub(super) source_action: Option<Type>,
}

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
    pub(super) reapply: bool,
    pub(super) reapplication_key: Option<Path>,
    pub(super) authored_actions: Option<AuthoredBodyActions>,
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
    mode: Once<CorrectionMode>,
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
    reapply: Once<bool>,
    reapplication_key: Once<Path>,
    program: Once<Type>,
    inverse_action: Once<Type>,
    source_action: Once<Type>,
}

impl CorrectionDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut keys = Keys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let mode_span = keys.mode.span();
        let mode = keys
            .mode
            .optional()
            .unwrap_or(CorrectionMode::RecordedInverse);
        let identity = resolve_identity(keys.namespace, keys.identity, missing)?;
        let demand = resolve_demand(
            keys.demand,
            keys.preimage_fields,
            keys.preimage_bytes,
            missing,
        )?;
        let source_body = keys.source_body.optional();
        if mode == CorrectionMode::AuthoredBody && source_body.is_none() {
            return Err(syn::Error::new(
                mode_span.unwrap_or(missing),
                "`mode = authored_body` requires `source_body = fn_path`",
            ));
        }
        let reapplication_key = keys.reapplication_key.optional();
        let reapply = keys.reapply.is_set() || reapplication_key.is_some();
        let authored_actions = resolve_authored_actions(
            mode,
            reapply,
            keys.program,
            keys.inverse_action,
            keys.source_action,
        )?;
        Ok(Self {
            schema: keys.schema.required(missing, DERIVE)?,
            identity,
            source: keys.source.required(missing, DERIVE)?,
            inverse: keys.inverse.required(missing, DERIVE)?,
            disclosure: keys.disclosure.required(missing, DERIVE)?,
            source_body,
            source_definition: keys.source_definition.optional(),
            disclosure_parameters: keys.disclosure_parameters.optional(),
            demand,
            derive_fn: keys.derive_fn.required(missing, DERIVE)?,
            inverse_key: keys.inverse_key.optional(),
            capability: keys.capability.optional(),
            reapply,
            reapplication_key,
            authored_actions,
        })
    }
}

impl Keys {
    fn new() -> Self {
        Self {
            mode: Once::new("mode"),
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
            reapply: Once::new("reapply"),
            reapplication_key: Once::new("reapplication_key"),
            program: Once::new("program"),
            inverse_action: Once::new("inverse_action"),
            source_action: Once::new("source_action"),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "mode" => self.mode.set(meta, parse_mode(meta)?),
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
            "reapply" => self.reapply.set(meta, flag(meta)?),
            "reapplication_key" => self.reapplication_key.set(meta, value(meta)?),
            "program" => self.program.set(meta, value(meta)?),
            "inverse_action" => self.inverse_action.set(meta, value(meta)?),
            "source_action" => self.source_action.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, KEYS)),
        }
    }
}

fn parse_mode(meta: &ParseNestedMeta<'_>) -> syn::Result<CorrectionMode> {
    let ident: Ident = meta.value()?.parse()?;
    match ident.unraw().to_string().as_str() {
        "recorded_inverse" => Ok(CorrectionMode::RecordedInverse),
        "authored_body" => Ok(CorrectionMode::AuthoredBody),
        other => Err(syn::Error::new(
            ident.span(),
            format!("unknown `mode = {other}`; expected `recorded_inverse` or `authored_body`"),
        )),
    }
}

fn resolve_authored_actions(
    mode: CorrectionMode,
    reapply: bool,
    program: Once<Type>,
    inverse_action: Once<Type>,
    source_action: Once<Type>,
) -> syn::Result<Option<AuthoredBodyActions>> {
    let is_authored = mode == CorrectionMode::AuthoredBody;
    let role = "a `mode = recorded_inverse` correction";
    program.refuse_unless(is_authored, role)?;
    inverse_action.refuse_unless(is_authored, role)?;
    source_action.refuse_unless(is_authored, role)?;
    let any_span = program
        .span()
        .or(inverse_action.span())
        .or(source_action.span());
    let Some(span) = any_span else {
        return Ok(None);
    };
    let program = program.required(span, DERIVE)?;
    let inverse_action = inverse_action.required(span, DERIVE)?;
    if let Some(src_span) = source_action.span()
        && !reapply
    {
        return Err(syn::Error::new(
            src_span,
            "`source_action` requires `reapply` (or `reapplication_key = fn`)",
        ));
    }
    Ok(Some(AuthoredBodyActions {
        program,
        inverse_action,
        source_action: source_action.optional(),
    }))
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
