//! Projection keys, slots and the uniqueness their generated names require.
use super::DERIVE;
use crate::case::pascal_case;
use crate::grammar::{
    member_name, option_inner, selection_column, unknown_key, value, visit_keys, Once,
};
use std::collections::BTreeSet;
use syn::{DeriveInput, Field, FieldsNamed, Ident, LitStr, Path, Type};

const STRUCT_KEYS: &[&str] = &["schema", "root", "namespace"];
const FIELD_KEYS: &[&str] = &["field", "column", "name", "slot"];

pub(super) struct ProjectionDeclaration {
    pub(super) schema: Type,
    pub(super) root: Type,
    pub(super) slots: Vec<Slot>,
}

pub(super) struct Slot {
    pub(super) member: Ident,
    /// The output name the query result carries.
    pub(super) output: LitStr,
    /// The slot marker type, named after the output.
    pub(super) marker: Ident,
    pub(super) selection: Type,
    pub(super) column: LitStr,
    pub(super) identity: SlotIdentity,
    /// The row field's declared type, which the projected value must match.
    pub(super) value: Type,
    pub(super) optional: bool,
}

pub(super) enum SlotIdentity {
    Derived(Path),
    Explicit(LitStr),
}

impl ProjectionDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: &FieldsNamed) -> syn::Result<Self> {
        let mut schema = Once::<Type>::new("schema");
        let mut root = Once::<Type>::new("root");
        let mut namespace = Once::<Path>::new("namespace");
        visit_keys(&input.attrs, |key, meta| match key {
            "schema" => schema.set(meta, value(meta)?),
            "root" => root.set(meta, value(meta)?),
            "namespace" => namespace.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        })?;
        let namespace = namespace.optional();
        if fields.named.is_empty() {
            return Err(syn::Error::new(
                input.ident.span(),
                "a projection needs at least one field",
            ));
        }
        let slots = fields
            .named
            .iter()
            .map(|field| slot(field, namespace.as_ref()))
            .collect::<syn::Result<Vec<_>>>()?;
        distinct(&slots)?;
        Ok(Self {
            schema: schema.required(input.ident.span(), DERIVE)?,
            root: root.required(input.ident.span(), DERIVE)?,
            slots,
        })
    }
}

fn slot(field: &Field, namespace: Option<&Path>) -> syn::Result<Slot> {
    let member = field.ident.clone().expect("named fields carry identifiers");
    let mut selection = Once::<Type>::new("field");
    let mut column = Once::<LitStr>::new("column");
    let mut output = Once::<LitStr>::new("name");
    let mut explicit = Once::<LitStr>::new("slot");
    visit_keys(&field.attrs, |key, meta| match key {
        "field" => selection.set(meta, value(meta)?),
        "column" => column.set(meta, value(meta)?),
        "name" => output.set(meta, value(meta)?),
        "slot" => explicit.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
    })?;
    let selection = selection.optional().ok_or_else(|| {
        syn::Error::new(
            member.span(),
            "missing `#[worth_query(field = Selection)]`: name the record field selection \
             this slot projects",
        )
    })?;
    let column = selection_column(&selection, column.optional(), member.span())?;
    let output = output
        .optional()
        .unwrap_or_else(|| LitStr::new(&member_name(&member), member.span()));
    let identity = identity(explicit.optional(), namespace, &member, &output)?;
    Ok(Slot {
        optional: option_inner(&field.ty).is_some(),
        value: field.ty.clone(),
        marker: marker(&output)?,
        member,
        output,
        selection,
        column,
        identity,
    })
}

/// An explicit `slot = ".."`, else the namespace's `slot.<name>`.
fn identity(
    explicit: Option<LitStr>,
    namespace: Option<&Path>,
    member: &Ident,
    output: &LitStr,
) -> syn::Result<SlotIdentity> {
    match (explicit, namespace) {
        (Some(identity), _) => Ok(SlotIdentity::Explicit(identity)),
        (None, Some(namespace)) => Ok(SlotIdentity::Derived(namespace.clone())),
        (None, None) => Err(syn::Error::new(
            member.span(),
            format!(
                "slot `{}` has no identity: add `namespace = NAMESPACE` to the struct \
                 or `slot = \"wire.identity\"` to this field",
                output.value()
            ),
        )),
    }
}

/// The slot marker type, named after the output in PascalCase.
fn marker(output: &LitStr) -> syn::Result<Ident> {
    syn::parse_str::<Ident>(&pascal_case(&output.value()))
        .map(|marker| Ident::new(&marker.to_string(), output.span()))
        .map_err(|_| {
            syn::Error::new(
                output.span(),
                format!(
                    "slot `{}` cannot name its slot marker; use letters, digits and `_`",
                    output.value()
                ),
            )
        })
}

/// Output names, their markers and explicit identities are each unique.
fn distinct(slots: &[Slot]) -> syn::Result<()> {
    let (mut outputs, mut markers, mut identities) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for slot in slots {
        if !outputs.insert(slot.output.value()) {
            return Err(duplicate(&slot.output, "slot"));
        }
        if !markers.insert(slot.marker.to_string()) {
            return Err(duplicate(&slot.output, "slot marker for"));
        }
        if let SlotIdentity::Explicit(identity) = &slot.identity
            && !identities.insert(identity.value())
        {
            return Err(duplicate(identity, "slot identity"));
        }
    }
    Ok(())
}

fn duplicate(at: &LitStr, what: &str) -> syn::Error {
    syn::Error::new(at.span(), format!("duplicate {what} `{}`", at.value()))
}
