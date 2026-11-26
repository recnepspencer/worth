//! The one scope every action declares: a marked input field, the principal's
//! field, a scope value function, or a constant scope value.
use super::super::DERIVE;
use crate::grammar::{selection_column, unknown_key, value, visit_keys, Once};
use syn::{DeriveInput, Expr, FieldsNamed, Ident, LitStr, Path, Type};

const FIELD_KEYS: &[&str] = &["scope", "column"];

pub(in crate::action) enum Scope {
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
    /// One fixed scope value for every input, from `scope_const = expr`.
    Const {
        selection: Type,
        column: LitStr,
        value: Expr,
    },
}

impl Scope {
    pub(in crate::action) fn selected_column(&self) -> (&Type, &LitStr) {
        match self {
            Self::Field {
                selection, column, ..
            }
            | Self::Principal { selection, column }
            | Self::Value {
                selection, column, ..
            }
            | Self::Const {
                selection, column, ..
            } => (selection, column),
        }
    }
}

/// Struct-level scope keys, each written at most once.
pub(super) struct StructScopeKeys {
    pub(super) principal: Once<Type>,
    pub(super) selection: Once<Type>,
    pub(super) function: Once<Path>,
    pub(super) constant: Once<Expr>,
    pub(super) column: Once<LitStr>,
}

impl StructScopeKeys {
    pub(super) fn new() -> Self {
        Self {
            principal: Once::new("principal_scope"),
            selection: Once::new("scope"),
            function: Once::new("scope_value"),
            constant: Once::new("scope_const"),
            column: Once::new("scope_column"),
        }
    }

    pub(super) fn resolve(self) -> syn::Result<Option<Scope>> {
        let Self {
            principal,
            selection,
            function,
            constant,
            column,
        } = self;
        let value_span = selection.span().or(function.span()).or(constant.span());
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
        if let (Some(_), Some(conflict)) = (function.span(), constant.span()) {
            return Err(syn::Error::new(
                conflict,
                "declare `scope_value = function` or `scope_const = value`, not both",
            ));
        }
        let selection = selection.required(span, DERIVE)?;
        let column = selection_column(&selection, column.optional(), span)?;
        if let Some(value) = constant.optional() {
            return Ok(Some(Scope::Const {
                selection,
                column,
                value,
            }));
        }
        let function = function.required(span, DERIVE)?;
        Ok(Some(Scope::Value {
            selection,
            column,
            function,
        }))
    }
}

/// The action's one scope, supplied by its declaration or a marked field.
pub(super) fn scope(
    input: &DeriveInput,
    fields: Option<&FieldsNamed>,
    mut scope: Option<Scope>,
) -> syn::Result<Scope> {
    for field in fields.iter().flat_map(|fields| &fields.named) {
        let Some(member) = field.ident.clone() else {
            continue;
        };
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
        scope = Some(Scope::Field {
            member,
            value: field.ty.clone(),
            selection,
            column,
        });
    }
    scope.ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "missing action scope: mark the field carrying it \
             `#[worth_query(scope = Selection)]`, declare `principal_scope = Selection`, \
             or declare `scope = Selection` with `scope_value = function` or `scope_const = value`",
        )
    })
}
