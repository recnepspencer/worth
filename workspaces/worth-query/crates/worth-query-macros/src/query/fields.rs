//! Field-level scope and named-parameter parsing for `#[derive(WorthQuery)]`.
use super::grammar::{QueryParam, QueryScope};
use super::DERIVE;
use crate::case::pascal_case;
use crate::grammar::{flag, member_name, selection_column, unknown_key, value, visit_keys, Once};
use std::collections::BTreeSet;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Field, FieldsNamed, Ident, LitStr, Type};

const FIELD_KEYS: &[&str] = &[
    "scope", "column", "param", "name", "binding", "selector", "skip",
];
const RESERVED_SELECTORS: &[&str] = &["reference", "definition", "definition_with"];

struct FieldKeys {
    selection: Once<Type>,
    column: Once<LitStr>,
    param: Once<bool>,
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
            "scope" => self.selection.set(meta, value(meta)?),
            "column" => self.column.set(meta, value(meta)?),
            "param" => self.param.set(meta, flag(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "binding" => self.binding.set(meta, value(meta)?),
            "selector" => self.selector.set(meta, value(meta)?),
            "skip" => self.skip.set(meta, flag(meta)?),
            _ => Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
        }
    }

    fn refuse_param_keys(&self, role: &str) -> syn::Result<()> {
        self.param.refuse_unless(false, role)?;
        self.name.refuse_unless(false, role)?;
        self.binding.refuse_unless(false, role)?;
        self.selector.refuse_unless(false, role)?;
        Ok(())
    }
}

pub(super) fn parse_fields(
    input: &DeriveInput,
    fields: Option<&FieldsNamed>,
    mut scope: Option<QueryScope>,
    has_parameters_fn: bool,
) -> syn::Result<(QueryScope, Vec<QueryParam>)> {
    let mut params = Vec::new();
    if let Some(fields) = fields {
        for field in &fields.named {
            parse_field(field, &mut scope, &mut params, has_parameters_fn)?;
        }
    }
    let scope = scope.ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "missing query scope: mark the field carrying it \
             `#[worth_query(scope = Selection)]`, declare `principal_scope = Selection`, \
             or declare `scope = Selection, scope_value = function`",
        )
    })?;
    distinct_params(&params)?;
    Ok((scope, params))
}

fn parse_field(
    field: &Field,
    scope: &mut Option<QueryScope>,
    params: &mut Vec<QueryParam>,
    has_parameters_fn: bool,
) -> syn::Result<()> {
    let member = field.ident.clone().expect("named fields carry identifiers");
    let keys = FieldKeys::parse(field)?;
    if let Some(span) = keys.selection.span() {
        keys.refuse_param_keys("a `scope` field")?;
        keys.skip.refuse_unless(false, "a `scope` field")?;
        if scope.is_some() {
            return Err(syn::Error::new(span, "a query has exactly one scope"));
        }
        let selection = keys.selection.optional().expect("scope was set");
        let column = selection_column(&selection, keys.column.optional(), member.span())?;
        *scope = Some(QueryScope::Field {
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
    let wire_name = keys
        .name
        .optional()
        .unwrap_or_else(|| LitStr::new(&member_name(&member), member.span()));
    let marker = param_marker(&wire_name)?;
    params.push(QueryParam {
        selector: keys.selector.optional().unwrap_or_else(|| member.clone()),
        value_ty: field.ty.clone(),
        member,
        wire_name,
        marker,
        binding: keys.binding.optional(),
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
