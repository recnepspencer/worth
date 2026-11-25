//! Field-level scope, managed-index probe, and named-parameter parsing for `#[derive(WorthQuery)]`.
use super::grammar::{QueryParam, QueryScope, StructScope};
use super::DERIVE;
use crate::case::pascal_case;
use crate::grammar::{
    flag, member_name, option_inner, selection_column, unknown_key, value, visit_keys, Once,
};
use std::collections::BTreeSet;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Field, FieldsNamed, Ident, LitStr, Token, Type};

const FIELD_KEYS: &[&str] = &[
    "scope", "column", "param", "probe", "optional", "name", "binding", "selector", "skip",
];
const RESERVED_SELECTORS: &[&str] = &["reference", "definition", "definition_with"];

struct FieldKeys {
    selection: Once<Option<Type>>,
    column: Once<LitStr>,
    param: Once<bool>,
    probe: Once<bool>,
    optional: Once<bool>,
    name: Once<LitStr>,
    binding: Once<Type>,
    selector: Once<Ident>,
    skip: Once<bool>,
}

impl FieldKeys {
    fn parse(field: &Field) -> syn::Result<Self> {
        let mut keys = Self {
            selection: Once::new("scope"),
            column: Once::new("column"),
            param: Once::new("param"),
            probe: Once::new("probe"),
            optional: Once::new("optional"),
            name: Once::new("name"),
            binding: Once::new("binding"),
            selector: Once::new("selector"),
            skip: Once::new("skip"),
        };
        visit_keys(&field.attrs, |key, meta| keys.read(key, meta))?;
        Ok(keys)
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "scope" => {
                if meta.input.is_empty() || meta.input.peek(Token![,]) {
                    self.selection.set(meta, None)
                } else {
                    self.selection.set(meta, Some(value(meta)?))
                }
            }
            "column" => self.column.set(meta, value(meta)?),
            "param" => self.param.set(meta, flag(meta)?),
            "probe" => self.probe.set(meta, flag(meta)?),
            "optional" => self.optional.set(meta, flag(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "binding" => self.binding.set(meta, value(meta)?),
            "selector" => self.selector.set(meta, value(meta)?),
            "skip" => self.skip.set(meta, flag(meta)?),
            _ => Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
        }
    }

    fn refuse_param_keys(&self, role: &str) -> syn::Result<()> {
        self.param.refuse_unless(false, role)?;
        self.probe.refuse_unless(false, role)?;
        self.optional.refuse_unless(false, role)?;
        self.name.refuse_unless(false, role)?;
        self.binding.refuse_unless(false, role)?;
        self.selector.refuse_unless(false, role)?;
        Ok(())
    }
}

struct ScopeAccumulator {
    resolved: Option<QueryScope>,
    struct_selection: Option<(Type, LitStr, proc_macro2::Span)>,
}

impl ScopeAccumulator {
    fn from_struct(declared: Option<StructScope>) -> Self {
        match declared {
            Some(StructScope::Resolved(scope)) => Self {
                resolved: Some(scope),
                struct_selection: None,
            },
            Some(StructScope::FieldSelection {
                selection,
                column,
                span,
            }) => Self {
                resolved: None,
                struct_selection: Some((selection, column, span)),
            },
            None => Self {
                resolved: None,
                struct_selection: None,
            },
        }
    }

    fn finish(self, input: &DeriveInput) -> syn::Result<QueryScope> {
        if let Some(scope) = self.resolved {
            return Ok(scope);
        }
        if let Some((_, _, span)) = self.struct_selection {
            return Err(syn::Error::new(
                span,
                format!("#[derive({DERIVE})] requires `#[worth_query(scope_value = ..)]`"),
            ));
        }
        Err(syn::Error::new(
            input.ident.span(),
            "missing query scope: mark the field carrying it \
             `#[worth_query(scope = Selection)]`, declare `principal_scope = Selection`, \
             or declare `scope = Selection, scope_value = function`",
        ))
    }
}

pub(super) fn parse_fields(
    input: &DeriveInput,
    fields: Option<&FieldsNamed>,
    declared_scope: Option<StructScope>,
    has_parameters_fn: bool,
    has_managed_index: bool,
) -> syn::Result<(QueryScope, Vec<QueryParam>)> {
    let mut scope = ScopeAccumulator::from_struct(declared_scope);
    let mut params = Vec::new();
    let mut probe_count = 0usize;
    if let Some(fields) = fields {
        for field in &fields.named {
            parse_field(
                field,
                &mut scope,
                &mut params,
                has_parameters_fn,
                has_managed_index,
                &mut probe_count,
            )?;
        }
    }
    let scope = scope.finish(input)?;
    if has_managed_index && probe_count == 0 {
        return Err(syn::Error::new(
            input.ident.span(),
            "`managed_index` requires one field marked `#[worth_query(probe)]`",
        ));
    }
    distinct_params(&params)?;
    Ok((scope, params))
}

fn parse_field(
    field: &Field,
    scope: &mut ScopeAccumulator,
    params: &mut Vec<QueryParam>,
    has_parameters_fn: bool,
    has_managed_index: bool,
    probe_count: &mut usize,
) -> syn::Result<()> {
    let member = field.ident.clone().expect("named fields carry identifiers");
    let keys = FieldKeys::parse(field)?;
    if let Some(span) = keys.selection.span() {
        keys.refuse_param_keys("a `scope` field")?;
        keys.skip.refuse_unless(false, "a `scope` field")?;
        if scope.resolved.is_some() {
            return Err(syn::Error::new(span, "a query has exactly one scope"));
        }
        let field_selection = keys.selection.optional().expect("scope was set");
        let (selection, column) = match (field_selection, scope.struct_selection.take()) {
            (Some(_), Some(_)) => {
                return Err(syn::Error::new(span, "a query has exactly one scope"));
            }
            (Some(selection), None) => {
                let column = selection_column(&selection, keys.column.optional(), member.span())?;
                (selection, column)
            }
            (None, Some((selection, struct_column, _))) => {
                let column = match keys.column.optional() {
                    Some(explicit) => explicit,
                    None => struct_column,
                };
                (selection, column)
            }
            (None, None) => {
                return Err(syn::Error::new(
                    span,
                    "`#[worth_query(scope)]` requires `scope = Selection` on the field or on the struct",
                ));
            }
        };
        scope.resolved = Some(QueryScope::Field {
            member,
            value_ty: field.ty.clone(),
            selection,
            column,
        });
        return Ok(());
    }
    if let Some(span) = keys.column.span() {
        return Err(syn::Error::new(
            span,
            "`column` names the column of a `scope` field",
        ));
    }
    if keys.skip.is_set() {
        return keys.refuse_param_keys("a `skip` field");
    }
    if has_parameters_fn {
        return keys.refuse_param_keys("a query with `parameters_fn`");
    }
    let is_probe = keys.probe.is_set();
    let optional_inner = option_inner(&field.ty).cloned();
    if let Some(span) = keys.optional.span()
        && optional_inner.is_none()
    {
        return Err(syn::Error::new(
            span,
            "`optional` requires an `Option<T>` parameter field type",
        ));
    }
    if let Some(span) = keys.probe.span() {
        if !has_managed_index {
            return Err(syn::Error::new(
                span,
                "`probe` requires struct-level `#[worth_query(managed_index = Index)]`",
            ));
        }
        if optional_inner.is_some() {
            return Err(syn::Error::new(
                span,
                "managed-index `probe` field cannot be `Option<T>`",
            ));
        }
        keys.param.refuse_unless(false, "a `probe` field")?;
        *probe_count += 1;
        if *probe_count > 1 {
            return Err(syn::Error::new(
                span,
                "a query has at most one `probe` field",
            ));
        }
    }
    let wire_name = keys
        .name
        .optional()
        .unwrap_or_else(|| LitStr::new(&member_name(&member), member.span()));
    let marker = param_marker(&wire_name)?;
    params.push(QueryParam {
        selector: keys.selector.optional().unwrap_or_else(|| member.clone()),
        value_ty: field.ty.clone(),
        optional_inner,
        member,
        wire_name,
        marker,
        binding: keys.binding.optional(),
        is_probe,
    });
    Ok(())
}

fn param_marker(name: &LitStr) -> syn::Result<Ident> {
    let value = name.value();
    if value.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            "a query parameter name cannot be empty",
        ));
    }
    syn::parse_str::<Ident>(&pascal_case(&value))
        .map(|marker| Ident::new(&marker.to_string(), name.span()))
        .map_err(|_| {
            syn::Error::new(
                name.span(),
                format!(
                    "parameter `{value}` cannot name its marker type; use letters, digits and `_`"
                ),
            )
        })
}

fn distinct_params(params: &[QueryParam]) -> syn::Result<()> {
    let (mut names, mut markers, mut selectors) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for param in params {
        if !names.insert(param.wire_name.value()) {
            return Err(syn::Error::new(
                param.wire_name.span(),
                format!("duplicate parameter `{}`", param.wire_name.value()),
            ));
        }
        if !markers.insert(param.marker.to_string()) {
            return Err(syn::Error::new(
                param.wire_name.span(),
                format!(
                    "duplicate parameter marker for `{}`",
                    param.wire_name.value()
                ),
            ));
        }
        let selector = member_name(&param.selector);
        if RESERVED_SELECTORS.contains(&selector.as_str()) || !selectors.insert(selector.clone()) {
            return Err(syn::Error::new(
                param.selector.span(),
                format!(
                    "parameter selector `{selector}` conflicts with an existing query item; \
                     rename it with `selector = ..`"
                ),
            ));
        }
    }
    Ok(())
}
