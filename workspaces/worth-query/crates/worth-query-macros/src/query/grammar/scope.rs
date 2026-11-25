//! Scope declaration parsing for `#[derive(WorthQuery)]`.
use super::super::DERIVE;
use crate::grammar::{selection_column, Once};
use proc_macro2::Span;
use syn::{Ident, LitStr, Path, Type};

pub(in crate::query) enum StructScope {
    Resolved(QueryScope),
    FieldSelection {
        selection: Type,
        column: LitStr,
        span: Span,
    },
}

pub(in crate::query) enum QueryScope {
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
    pub(in crate::query) fn selected_column(&self) -> (&Type, &LitStr) {
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

pub(super) fn resolve_struct_scope(
    principal: Once<Type>,
    selection: Once<Type>,
    function: Once<Path>,
    column: Once<LitStr>,
) -> syn::Result<Option<StructScope>> {
    let value_span = selection.span().or(function.span());
    if let Some(span) = principal.span() {
        if let Some(conflict) = value_span {
            return Err(syn::Error::new(conflict, "a query has exactly one scope"));
        }
        let selection = principal.required(span, DERIVE)?;
        let column = selection_column(&selection, column.optional(), span)?;
        return Ok(Some(StructScope::Resolved(QueryScope::Principal {
            selection,
            column,
        })));
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
    let column = selection_column(&selection, column.optional(), span)?;
    match function.optional() {
        Some(function) => Ok(Some(StructScope::Resolved(QueryScope::Value {
            selection,
            column,
            function,
        }))),
        None => Ok(Some(StructScope::FieldSelection {
            selection,
            column,
            span,
        })),
    }
}
