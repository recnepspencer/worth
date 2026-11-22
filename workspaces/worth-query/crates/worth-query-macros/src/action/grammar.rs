//! Action keys, capability lists and the one scope every action declares.
use super::DERIVE;
use crate::grammar::{
    flag, selection_column, token_group, type_list, unknown_key, value, visit_keys, Once,
};
use proc_macro2::{Span, TokenStream};
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, FieldsNamed, Ident, LitStr, Path, Type};

/// Inline capability families `worth_query_operation!` accepts.
const CAPABILITIES: &[&str] = &[
    "reads",
    "writes",
    "creates",
    "deletes",
    "links",
    "unlinks",
    "emits",
    "expects_version",
    "expects_fact",
];
const STRUCT_KEYS: &[&str] = &[
    "schema",
    "context",
    "namespace",
    "input",
    "operation",
    "binding",
    "delivery",
    "result",
    "decision",
    "outputs",
    "body",
    "program",
    "access",
    "principal_scope",
    "reads, writes, creates, deletes, links, unlinks, emits, expects_version, expects_fact",
];
const FIELD_KEYS: &[&str] = &["scope", "column"];

pub(super) struct ActionDeclaration {
    pub(super) schema: Type,
    pub(super) context: Type,
    pub(super) namespace: Path,
    pub(super) input: Ident,
    pub(super) operation: Ident,
    pub(super) binding: Ident,
    pub(super) delivery: Option<Ident>,
    pub(super) result: Type,
    pub(super) decision: Type,
    pub(super) outputs: Option<Type>,
    pub(super) body: bool,
    pub(super) program: bool,
    pub(super) access: Option<TokenStream>,
    pub(super) capabilities: Vec<(Ident, Vec<Type>)>,
    pub(super) scope: Scope,
}

pub(super) enum Scope {
    /// The input field carrying the scope value.
    Field {
        member: Ident,
        value: Type,
        selection: Type,
        column: LitStr,
    },
    /// The authenticated principal's own field.
    Principal { selection: Type, column: LitStr },
}

/// Every struct-level key, each written at most once.
struct Keys {
    schema: Once<Type>,
    context: Once<Type>,
    namespace: Once<Path>,
    input: Once<Ident>,
    operation: Once<Ident>,
    binding: Once<Ident>,
    delivery: Once<Ident>,
    result: Once<Type>,
    decision: Once<Type>,
    outputs: Once<Type>,
    access: Once<TokenStream>,
    principal_scope: Once<Type>,
    body: bool,
    program: bool,
    capabilities: Vec<(Ident, Vec<Type>)>,
}

impl ActionDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: &FieldsNamed) -> syn::Result<Self> {
        let mut keys = Keys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let principal_span = keys.principal_scope.span();
        let principal = keys.principal_scope.optional().zip(principal_span);
        Ok(Self {
            scope: scope(input, fields, principal)?,
            schema: keys.schema.required(missing, DERIVE)?,
            context: keys.context.required(missing, DERIVE)?,
            namespace: keys.namespace.required(missing, DERIVE)?,
            input: keys.input.required(missing, DERIVE)?,
            operation: keys.operation.required(missing, DERIVE)?,
            binding: keys.binding.required(missing, DERIVE)?,
            delivery: keys.delivery.optional(),
            result: keys.result.required(missing, DERIVE)?,
            decision: keys.decision.required(missing, DERIVE)?,
            outputs: keys.outputs.optional(),
            body: keys.body,
            program: keys.program,
            access: keys.access.optional(),
            capabilities: keys.capabilities,
        })
    }
}

impl Keys {
    fn new() -> Self {
        Self {
            schema: Once::new("schema"),
            context: Once::new("context"),
            namespace: Once::new("namespace"),
            input: Once::new("input"),
            operation: Once::new("operation"),
            binding: Once::new("binding"),
            delivery: Once::new("delivery"),
            result: Once::new("result"),
            decision: Once::new("decision"),
            outputs: Once::new("outputs"),
            access: Once::new("access"),
            principal_scope: Once::new("principal_scope"),
            body: false,
            program: false,
            capabilities: Vec::new(),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "schema" => self.schema.set(meta, value(meta)?),
            "context" => self.context.set(meta, value(meta)?),
            "namespace" => self.namespace.set(meta, value(meta)?),
            "input" => self.input.set(meta, value(meta)?),
            "operation" => self.operation.set(meta, value(meta)?),
            "binding" => self.binding.set(meta, value(meta)?),
            "delivery" => self.delivery.set(meta, value(meta)?),
            "result" => self.result.set(meta, value(meta)?),
            "decision" => self.decision.set(meta, value(meta)?),
            "outputs" => self.outputs.set(meta, value(meta)?),
            "access" => self.access.set(meta, token_group(meta)?),
            "principal_scope" => self.principal_scope.set(meta, value(meta)?),
            "body" => flag(meta).map(|body| self.body = body),
            "program" => flag(meta).map(|program| self.program = program),
            _ if CAPABILITIES.contains(&key) => self.capability(meta, key),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }

    /// `reads(Task, ..)` and the other inline capability lists, once each.
    fn capability(&mut self, meta: &ParseNestedMeta<'_>, key: &str) -> syn::Result<()> {
        if self.capabilities.iter().any(|(kind, _)| kind == key) {
            return Err(meta.error(format!("duplicate `{key}` key")));
        }
        let kind = meta
            .path
            .get_ident()
            .cloned()
            .expect("capability keys are identifiers");
        self.capabilities.push((kind, type_list(meta)?));
        Ok(())
    }
}

/// The action's one scope: a marked field or the principal's own field.
fn scope(
    input: &DeriveInput,
    fields: &FieldsNamed,
    principal: Option<(Type, Span)>,
) -> syn::Result<Scope> {
    let mut scope = match principal {
        Some((selection, span)) => {
            let column = selection_column(&selection, None, span)?;
            Some((Scope::Principal { selection, column }, span))
        }
        None => None,
    };
    for field in &fields.named {
        let member = field.ident.clone().expect("named fields carry identifiers");
        let mut selection = Once::<Type>::new("scope");
        let mut column = Once::<LitStr>::new("column");
        visit_keys(&field.attrs, |key, meta| match key {
            "scope" => selection.set(meta, value(meta)?),
            "column" => column.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
        })?;
        let marked = selection.span();
        if let (None, Some(span)) = (marked, column.span()) {
            return Err(syn::Error::new(
                span,
                "`column` names the column of a `scope` field",
            ));
        }
        let (Some(span), Some(selection)) = (marked, selection.optional()) else {
            continue;
        };
        if scope.is_some() {
            return Err(syn::Error::new(span, "an action has exactly one scope"));
        }
        let column = selection_column(&selection, column.optional(), member.span())?;
        let value = field.ty.clone();
        let declared = Scope::Field {
            member,
            value,
            selection,
            column,
        };
        scope = Some((declared, span));
    }
    scope.map(|(scope, _)| scope).ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "missing action scope: mark the field carrying it \
             `#[worth_query(scope = Selection)]` or declare `principal_scope = Selection`",
        )
    })
}
