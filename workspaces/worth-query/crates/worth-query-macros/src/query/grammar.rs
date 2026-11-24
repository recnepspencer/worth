//! Struct-level keys, scope selection, and definition defaults for `#[derive(WorthQuery)]`.
use super::fields::parse_fields;
use super::DERIVE;
use crate::case::snake_case;
use crate::grammar::{flag, selection_column, unknown_key, value, visit_keys, Once};
use proc_macro2::Span;
use quote::format_ident;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, FieldsNamed, Ident, LitStr, Path, Type};

const STRUCT_KEYS: &[&str] = &[
    "schema",
    "context",
    "namespace",
    "query",
    "input",
    "binding",
    "result",
    "parameters",
    "parameters_fn",
    "bindings",
    "name",
    "scope_name",
    "principal_scope",
    "scope",
    "scope_value",
    "scope_column",
    "limits",
    "validate",
    "root",
    "cardinality",
    "disclosure",
    "basis_support",
    "lanes",
    "ability",
    "explicit_shape",
];

pub(super) struct QueryDeclaration {
    pub(super) schema: Type,
    pub(super) context: Type,
    pub(super) namespace: Path,
    pub(super) query: Ident,
    pub(super) input: Ident,
    pub(super) binding: Ident,
    pub(super) result: Path,
    pub(super) parameters_binding: Option<Path>,
    pub(super) parameters_fn: Option<Path>,
    pub(super) bindings: Option<Type>,
    pub(super) query_name: LitStr,
    pub(super) scope_name: Option<LitStr>,
    pub(super) limits: Expr,
    pub(super) validate: Option<Path>,
    pub(super) scope: QueryScope,
    pub(super) params: Vec<QueryParam>,
    pub(super) definition: Option<DefinitionDefaults>,
}

pub(super) enum QueryScope {
    Field {
        member: Ident,
        value_ty: Type,
        selection: Type,
        column: LitStr,
    },
    Principal {
        selection: Type,
        column: LitStr,
    },
    Value {
        selection: Type,
        column: LitStr,
        function: Path,
    },
}

impl QueryScope {
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

pub(super) struct QueryParam {
    pub(super) member: Ident,
    pub(super) value_ty: Type,
    pub(super) wire_name: LitStr,
    pub(super) marker: Ident,
    pub(super) selector: Ident,
    pub(super) binding: Option<Type>,
}

pub(super) struct DefinitionDefaults {
    pub(super) root: Type,
    pub(super) cardinality: Option<Expr>,
    pub(super) disclosure: Option<Expr>,
    pub(super) basis_support: Option<Expr>,
    pub(super) lanes: Option<Expr>,
    pub(super) ability: Option<Expr>,
    pub(super) explicit_shape: bool,
}

struct StructKeys {
    schema: Once<Type>,
    context: Once<Type>,
    namespace: Once<Path>,
    query: Once<Ident>,
    input: Once<Ident>,
    binding: Once<Ident>,
    result: Once<Path>,
    parameters: Once<Path>,
    parameters_fn: Once<Path>,
    bindings: Once<Type>,
    name: Once<LitStr>,
    scope_name: Once<LitStr>,
    principal_scope: Once<Type>,
    scope: Once<Type>,
    scope_value: Once<Path>,
    scope_column: Once<LitStr>,
    limits: Once<Expr>,
    validate: Once<Path>,
    root: Once<Type>,
    cardinality: Once<Expr>,
    disclosure: Once<Expr>,
    basis_support: Once<Expr>,
    lanes: Once<Expr>,
    ability: Once<Expr>,
    explicit_shape: Once<bool>,
}

impl QueryDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: Option<&FieldsNamed>) -> syn::Result<Self> {
        let mut keys = StructKeys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let declared_scope = keys.take_scope()?;
        let definition = keys.take_definition()?;
        let parameters_fn = keys.parameters_fn.optional();
        let (scope, params) = parse_fields(input, fields, declared_scope, parameters_fn.is_some())?;
        let input_binding = keys
            .input
            .optional()
            .unwrap_or_else(|| format_ident!("{}Input", input.ident, span = input.ident.span()));
        let query_name = keys.name.optional().unwrap_or_else(|| {
            LitStr::new(&snake_case(&input.ident.to_string()), input.ident.span())
        });
        Ok(Self {
            schema: keys.schema.required(missing, DERIVE)?,
            context: keys.context.required(missing, DERIVE)?,
            namespace: keys.namespace.required(missing, DERIVE)?,
            query: keys.query.required(missing, DERIVE)?,
            input: input_binding,
            binding: keys.binding.required(missing, DERIVE)?,
            result: keys.result.required(missing, DERIVE)?,
            parameters_binding: keys.parameters.optional(),
            parameters_fn,
            bindings: keys.bindings.optional(),
            query_name,
            scope_name: keys.scope_name.optional(),
            limits: keys.limits.required(missing, DERIVE)?,
            validate: keys.validate.optional(),
            scope,
            params,
            definition,
        })
    }
}

impl StructKeys {
    fn new() -> Self {
        Self {
            schema: Once::new("schema"),
            context: Once::new("context"),
            namespace: Once::new("namespace"),
            query: Once::new("query"),
            input: Once::new("input"),
            binding: Once::new("binding"),
            result: Once::new("result"),
            parameters: Once::new("parameters"),
            parameters_fn: Once::new("parameters_fn"),
            bindings: Once::new("bindings"),
            name: Once::new("name"),
            scope_name: Once::new("scope_name"),
            principal_scope: Once::new("principal_scope"),
            scope: Once::new("scope"),
            scope_value: Once::new("scope_value"),
            scope_column: Once::new("scope_column"),
            limits: Once::new("limits"),
            validate: Once::new("validate"),
            root: Once::new("root"),
            cardinality: Once::new("cardinality"),
            disclosure: Once::new("disclosure"),
            basis_support: Once::new("basis_support"),
            lanes: Once::new("lanes"),
            ability: Once::new("ability"),
            explicit_shape: Once::new("explicit_shape"),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "schema" => self.schema.set(meta, value(meta)?),
            "context" => self.context.set(meta, value(meta)?),
            "namespace" => self.namespace.set(meta, value(meta)?),
            "query" => self.query.set(meta, value(meta)?),
            "input" => self.input.set(meta, value(meta)?),
            "binding" => self.binding.set(meta, value(meta)?),
            "result" => self.result.set(meta, value(meta)?),
            "parameters" => self.parameters.set(meta, value(meta)?),
            "parameters_fn" => self.parameters_fn.set(meta, value(meta)?),
            "bindings" => self.bindings.set(meta, value(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "scope_name" => self.scope_name.set(meta, value(meta)?),
            "principal_scope" => self.principal_scope.set(meta, value(meta)?),
            "scope" => self.scope.set(meta, value(meta)?),
            "scope_value" => self.scope_value.set(meta, value(meta)?),
            "scope_column" => self.scope_column.set(meta, value(meta)?),
            "limits" => self.limits.set(meta, value(meta)?),
            "validate" => self.validate.set(meta, validator(meta)?),
            "root" => self.root.set(meta, value(meta)?),
            "cardinality" => self.cardinality.set(meta, value(meta)?),
            "disclosure" => self.disclosure.set(meta, value(meta)?),
            "basis_support" => self.basis_support.set(meta, value(meta)?),
            "lanes" => self.lanes.set(meta, value(meta)?),
            "ability" => self.ability.set(meta, value(meta)?),
            "explicit_shape" => self.explicit_shape.set(meta, flag(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }
    fn take_scope(&mut self) -> syn::Result<Option<QueryScope>> {
        let principal = std::mem::replace(&mut self.principal_scope, Once::new("principal_scope"));
        let selection = std::mem::replace(&mut self.scope, Once::new("scope"));
        let function = std::mem::replace(&mut self.scope_value, Once::new("scope_value"));
        let column = std::mem::replace(&mut self.scope_column, Once::new("scope_column"));
        let value_span = selection.span().or(function.span());
        if let Some(span) = principal.span() {
            if let Some(conflict) = value_span {
                return Err(syn::Error::new(conflict, "a query has exactly one scope"));
            }
            let selection = principal.required(span, DERIVE)?;
            let column = selection_column(&selection, column.optional(), span)?;
            return Ok(Some(QueryScope::Principal { selection, column }));
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
        Ok(Some(QueryScope::Value {
            selection,
            column,
            function,
        }))
    }

    fn take_definition(&mut self) -> syn::Result<Option<DefinitionDefaults>> {
        let root = std::mem::replace(&mut self.root, Once::new("root"));
        let cardinality = std::mem::replace(&mut self.cardinality, Once::new("cardinality"));
        let disclosure = std::mem::replace(&mut self.disclosure, Once::new("disclosure"));
        let basis_support = std::mem::replace(&mut self.basis_support, Once::new("basis_support"));
        let lanes = std::mem::replace(&mut self.lanes, Once::new("lanes"));
        let ability = std::mem::replace(&mut self.ability, Once::new("ability"));
        let explicit_shape =
            std::mem::replace(&mut self.explicit_shape, Once::new("explicit_shape"));
        let has_root = root.is_set();
        let label = "a query without `root = Entity`";
        cardinality.refuse_unless(has_root, label)?;
        disclosure.refuse_unless(has_root, label)?;
        basis_support.refuse_unless(has_root, label)?;
        lanes.refuse_unless(has_root, label)?;
        ability.refuse_unless(has_root, label)?;
        explicit_shape.refuse_unless(has_root, label)?;
        Ok(root.optional().map(|root| DefinitionDefaults {
            root,
            cardinality: cardinality.optional(),
            disclosure: disclosure.optional(),
            basis_support: basis_support.optional(),
            lanes: lanes.optional(),
            ability: ability.optional(),
            explicit_shape: explicit_shape.is_set(),
        }))
    }
}

fn validator(meta: &ParseNestedMeta<'_>) -> syn::Result<Path> {
    value::<Path>(meta).map_err(|error| {
        syn::Error::new(
            error.span(),
            "`validate` takes a function path, such as `validate = rules::check_query`",
        )
    })
}

/// Checks if `expr` is a single bare identifier.
pub(super) fn bare_ident(expr: &Expr) -> Option<(&Ident, Span)> {
    let Expr::Path(path) = expr else {
        return None;
    };
    if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 {
        let seg = &path.path.segments[0];
        if seg.arguments.is_empty() {
            return Some((&seg.ident, seg.ident.span()));
        }
    }
    None
}
