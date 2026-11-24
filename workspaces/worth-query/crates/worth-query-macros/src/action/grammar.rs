//! Action keys, capability lists and the one scope every action declares.
use super::serde_names::{container_rename_rule, field_wire_name};
use super::DERIVE;
use crate::facade_paths::{declaration, fragment, host};
use crate::grammar::{
    flag, option_inner, selection_column, token_group, type_list, unknown_key, value, visit_keys,
    Once,
};
use proc_macro2::TokenStream;
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
    "scope",
    "scope_value",
    "scope_column",
    "validate",
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
    /// The input validator, forwarded as the input binding's `validate:`.
    pub(super) validate: Option<Path>,
    pub(super) body: bool,
    pub(super) program: bool,
    pub(super) access: Option<TokenStream>,
    pub(super) capabilities: Vec<(Ident, Vec<Type>)>,
    pub(super) scope: Scope,
    /// Named fields of the input struct, each receiving a typed `<field>_input()`.
    pub(super) input_fields: Vec<InputField>,
}

pub(super) struct InputField {
    pub(super) member: Ident,
    pub(super) wire_name: LitStr,
    /// The scalar value type (`T` for both `T` and `Option<T>`).
    pub(super) value_ty: Type,
    pub(super) optional: bool,
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
    /// Scope selected by the supplied typed value function.
    Value {
        selection: Type,
        column: LitStr,
        function: Path,
    },
}

impl Scope {
    pub(super) fn selected_column(&self) -> (&Type, &LitStr) {
        match self {
            Self::Field {
                selection, column, ..
            }
            | Self::Principal { selection, column }
            | Self::Value {
                selection, column, ..
            } => (selection, column),
        }
    }
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
    scope: Once<Type>,
    scope_value: Once<Path>,
    scope_column: Once<LitStr>,
    validate: Once<Path>,
    body: bool,
    program: bool,
    capabilities: Vec<(Ident, Vec<Type>)>,
}

impl ActionDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: Option<&FieldsNamed>) -> syn::Result<Self> {
        let mut keys = Keys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let declared_scope = struct_scope(
            keys.principal_scope,
            keys.scope,
            keys.scope_value,
            keys.scope_column,
        )?;
        let scope = scope(input, fields, declared_scope)?;
        let schema = keys.schema.required(missing, DERIVE)?;
        let (result, decision) = if keys.body {
            let (selection, _) = scope.selected_column();
            let (decl, host, frag) = (declaration(), host(), fragment());
            (
                keys.result.optional().unwrap_or_else(
                    || syn::parse_quote!(#decl::application_schema::UnitApplicationValueBinding),
                ),
                keys.decision.optional().unwrap_or_else(|| {
                    syn::parse_quote!(#host::primary_graph::WorthQueryOperationBodyDecision<
                        #schema, <#selection as #frag::RecordSelection>::Entity
                    >)
                }),
            )
        } else {
            (
                keys.result.required(missing, DERIVE)?,
                keys.decision.required(missing, DERIVE)?,
            )
        };
        let rename_rule = container_rename_rule(&input.attrs)?;
        let mut input_fields = Vec::new();
        if let Some(named) = fields {
            for field in &named.named {
                let Some(wire_name) = field_wire_name(field, rename_rule)? else {
                    continue;
                };
                let member = field.ident.clone().expect("named fields carry identifiers");
                let (optional, value_ty) = match option_inner(&field.ty) {
                    Some(inner) => (true, inner.clone()),
                    None => (false, field.ty.clone()),
                };
                input_fields.push(InputField {
                    member,
                    wire_name,
                    value_ty,
                    optional,
                });
            }
        }
        Ok(Self {
            scope,
            schema,
            context: keys.context.required(missing, DERIVE)?,
            namespace: keys.namespace.required(missing, DERIVE)?,
            input: keys.input.required(missing, DERIVE)?,
            operation: keys.operation.required(missing, DERIVE)?,
            binding: keys.binding.required(missing, DERIVE)?,
            delivery: keys.delivery.optional(),
            result,
            decision,
            outputs: keys.outputs.optional(),
            validate: keys.validate.optional(),
            body: keys.body,
            program: keys.program,
            access: keys.access.optional(),
            capabilities: keys.capabilities,
            input_fields,
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
            scope: Once::new("scope"),
            scope_value: Once::new("scope_value"),
            scope_column: Once::new("scope_column"),
            validate: Once::new("validate"),
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
            "scope" => self.scope.set(meta, value(meta)?),
            "scope_value" => self.scope_value.set(meta, value(meta)?),
            "scope_column" => self.scope_column.set(meta, value(meta)?),
            "validate" => self.validate.set(meta, validator(meta)?),
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

/// `validate = path::to::check`: a function path, refused at the value otherwise.
fn validator(meta: &ParseNestedMeta<'_>) -> syn::Result<Path> {
    value::<Path>(meta).map_err(|error| {
        syn::Error::new(
            error.span(),
            "`validate` takes a function path, such as `validate = rules::check_title`",
        )
    })
}

fn struct_scope(
    principal: Once<Type>,
    selection: Once<Type>,
    function: Once<Path>,
    column: Once<LitStr>,
) -> syn::Result<Option<Scope>> {
    let value_span = selection.span().or(function.span());
    if let Some(span) = principal.span() {
        if let Some(conflict) = value_span {
            return Err(syn::Error::new(conflict, "an action has exactly one scope"));
        }
        let selection = principal.required(span, DERIVE)?;
        let column = selection_column(&selection, column.optional(), span)?;
        return Ok(Some(Scope::Principal { selection, column }));
    }
    let Some(span) = value_span else {
        if let Some(col_span) = column.span() {
            return Err(syn::Error::new(
                col_span,
                "`scope_column` requires struct-level `principal_scope` or `scope`",
            ));
        }
        return Ok(None);
    };
    let selection = selection.required(span, DERIVE)?;
    let function = function.required(span, DERIVE)?;
    let column = selection_column(&selection, column.optional(), span)?;
    Ok(Some(Scope::Value {
        selection,
        column,
        function,
    }))
}

/// The action's one scope, supplied by its declaration or a marked field.
fn scope(
    input: &DeriveInput,
    fields: Option<&FieldsNamed>,
    mut scope: Option<Scope>,
) -> syn::Result<Scope> {
    if let Some(fields) = fields {
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
            scope = Some(declared);
        }
    }
    scope.ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "missing action scope: mark the field carrying it \
             `#[worth_query(scope = Selection)]`, declare `principal_scope = Selection`, \
             or declare `scope = Selection, scope_value = function`",
        )
    })
}
