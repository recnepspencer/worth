//! Projection keys, slots and the uniqueness their generated names require.
use super::DERIVE;
use crate::case::pascal_case;
use crate::grammar::{
    flag, member_name, option_inner, peeled, selection_column, unknown_key, value, vec_inner,
    visit_keys, Once,
};
use std::collections::BTreeSet;
use syn::{DeriveInput, Field, FieldsNamed, Ident, LitStr, Path, Type};

const STRUCT_KEYS: &[&str] = &["schema", "root", "namespace"];
const FIELD_KEYS: &[&str] = &[
    "field",
    "column",
    "relation",
    "reverse_from",
    "nested",
    "name",
    "group_name",
    "slot",
    "selector",
];
/// Inherent items every derived projection declares beside its selectors.
const GENERATED: &[&str] = &[
    "result_shape",
    "result_shape_builder",
    "project_row",
    "PROJECTS_PERSISTENT_NAMES",
];

pub(super) struct ProjectionDeclaration {
    pub(super) schema: Type,
    pub(super) root: Type,
    pub(super) namespace: Option<Path>,
    pub(super) slots: Vec<Slot>,
    /// Fields that project the row's own persistent name.
    pub(super) names: Vec<NameSlot>,
}

/// A field projecting the persistent name the row's root entity carries. It
/// is not a declared result slot, so it has no selector, marker or identity.
pub(super) struct NameSlot {
    pub(super) member: Ident,
    /// The field's declared type, which the projected name must match.
    pub(super) value: Type,
    /// `Option<..>`: an entity carrying no name projects `None`.
    pub(super) optional: bool,
    /// `group_name`: the name of the group, of the field's group kind, the
    /// row's own name belongs to.
    pub(super) group: bool,
}

enum Member {
    Slot(Slot),
    Name(NameSlot),
}

pub(super) struct Slot {
    pub(super) member: Ident,
    /// The query-generic selector function: the field name unless overridden.
    pub(super) selector: Ident,
    /// The output name the query result carries.
    pub(super) output: LitStr,
    /// The slot marker type, named after the output.
    pub(super) marker: Ident,
    pub(super) identity: SlotIdentity,
    /// The row field's declared type, which the projected value must match.
    pub(super) value: Type,
    pub(super) kind: SlotKind,
}

pub(super) enum SlotKind {
    /// A root-entity field read through its record field selection.
    Field {
        selection: Type,
        column: LitStr,
        optional: bool,
    },
    /// A traversed relation whose related rows project into a derived child.
    Relation(Traversal),
}

pub(super) struct Traversal {
    pub(super) relation: Type,
    /// The declaring entity of a relation traversed in reverse, toward the root.
    pub(super) reverse_from: Option<Type>,
    /// The structured value binding that identifies the nested result shape.
    pub(super) nested: Type,
    /// The derived projection each related row reads into.
    pub(super) child: Type,
    pub(super) cardinality: Cardinality,
}

/// The related-row count the field type admits: `T`, `Option<T>` or `Vec<T>`.
#[derive(Clone, Copy)]
pub(super) enum Cardinality {
    One,
    Optional,
    Many,
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
        let (mut slots, mut names) = (Vec::new(), Vec::new());
        for field in &fields.named {
            match member(field, namespace.as_ref())? {
                Member::Slot(slot) => slots.push(slot),
                Member::Name(name) => names.push(name),
            }
        }
        distinct(&slots)?;
        Ok(Self {
            schema: schema.required(input.ident.span(), DERIVE)?,
            root: root.required(input.ident.span(), DERIVE)?,
            namespace,
            slots,
            names,
        })
    }
}

/// The keys that choose what a slot reads: a field or a traversed relation.
struct KindKeys {
    selection: Once<Type>,
    column: Once<LitStr>,
    relation: Once<Type>,
    reverse_from: Once<Type>,
    nested: Once<Type>,
}

/// A bare `name` key projects the row's persistent name; `name = ".."` names
/// a slot's output.
fn member(field: &Field, namespace: Option<&Path>) -> syn::Result<Member> {
    let member = field.ident.clone().expect("named fields carry identifiers");
    let mut named = Once::<bool>::new("name");
    let mut grouped = Once::<bool>::new("group_name");
    let mut kind = KindKeys {
        selection: Once::new("field"),
        column: Once::new("column"),
        relation: Once::new("relation"),
        reverse_from: Once::new("reverse_from"),
        nested: Once::new("nested"),
    };
    let mut output = Once::<LitStr>::new("name");
    let mut explicit = Once::<LitStr>::new("slot");
    let mut selector = Once::<Ident>::new("selector");
    visit_keys(&field.attrs, |key, meta| match key {
        "field" => kind.selection.set(meta, value(meta)?),
        "column" => kind.column.set(meta, value(meta)?),
        "relation" => kind.relation.set(meta, value(meta)?),
        "reverse_from" => kind.reverse_from.set(meta, value(meta)?),
        "nested" => kind.nested.set(meta, value(meta)?),
        "name" if meta.input.peek(syn::Token![=]) => output.set(meta, value(meta)?),
        "name" => named.set(meta, flag(meta)?),
        "group_name" => grouped.set(meta, flag(meta)?),
        "slot" => explicit.set(meta, value(meta)?),
        "selector" => selector.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
    })?;
    if let (Some(span), true) = (grouped.span(), named.is_set()) {
        return Err(syn::Error::new(
            span,
            "a field projects either the row's `name` or its `group_name`, not both",
        ));
    }
    let group = grouped.is_set();
    if named.is_set() || group {
        let role = "a name slot, which projects the row's own persistent name";
        kind.selection.refuse_unless(false, role)?;
        kind.column.refuse_unless(false, role)?;
        kind.relation.refuse_unless(false, role)?;
        kind.reverse_from.refuse_unless(false, role)?;
        kind.nested.refuse_unless(false, role)?;
        output.refuse_unless(false, role)?;
        explicit.refuse_unless(false, role)?;
        selector.refuse_unless(false, role)?;
        return Ok(Member::Name(NameSlot {
            optional: option_inner(&field.ty).is_some(),
            value: field.ty.clone(),
            member,
            group,
        }));
    }
    let kind = slot_kind(kind, &member, &field.ty)?;
    let output = output
        .optional()
        .unwrap_or_else(|| LitStr::new(&member_name(&member), member.span()));
    let identity = identity(explicit.optional(), namespace, &member, &output)?;
    Ok(Member::Slot(Slot {
        selector: selector.optional().unwrap_or_else(|| member.clone()),
        value: field.ty.clone(),
        marker: marker(&output)?,
        member,
        output,
        identity,
        kind,
    }))
}

/// Exactly one of `field` and `relation`, with only the keys that one uses.
fn slot_kind(keys: KindKeys, member: &Ident, ty: &Type) -> syn::Result<SlotKind> {
    let KindKeys {
        selection,
        column,
        relation,
        reverse_from,
        nested,
    } = keys;
    if let (Some(span), true) = (relation.span(), selection.is_set()) {
        return Err(syn::Error::new(
            span,
            "a slot projects either a `field` or a `relation`, not both",
        ));
    }
    if let Some(span) = relation.span() {
        column.refuse_unless(false, "a relation slot")?;
        let nested = nested.optional().ok_or_else(|| {
            syn::Error::new(
                span,
                "a relation slot names the binding of its nested result shape: \
                 add `nested = Binding`",
            )
        })?;
        let relation = relation.optional().expect("the relation key was written");
        return Ok(SlotKind::Relation(traversal(
            relation,
            reverse_from.optional(),
            nested,
            ty,
        )));
    }
    reverse_from.refuse_unless(false, "a field slot")?;
    nested.refuse_unless(false, "a field slot")?;
    let selection = selection.optional().ok_or_else(|| {
        syn::Error::new(
            member.span(),
            "missing `#[worth_query(field = Selection)]`: name the record field selection \
             this slot projects, or `relation = Relation` for a traversed relation",
        )
    })?;
    let column = selection_column(&selection, column.optional(), member.span())?;
    Ok(SlotKind::Field {
        optional: option_inner(ty).is_some(),
        selection,
        column,
    })
}

/// `Vec<T>` traverses many related rows, `Option<T>` at most one, else one.
fn traversal(relation: Type, reverse_from: Option<Type>, nested: Type, ty: &Type) -> Traversal {
    let (cardinality, child) = match (vec_inner(ty), option_inner(ty)) {
        (Some(child), _) => (Cardinality::Many, child),
        (None, Some(child)) => (Cardinality::Optional, child),
        (None, None) => (Cardinality::One, ty),
    };
    Traversal {
        relation,
        reverse_from,
        nested,
        child: peeled(child).clone(),
        cardinality,
    }
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

/// Output names, their markers, selectors and explicit identities are each
/// unique, and no selector takes a generated function's name.
fn distinct(slots: &[Slot]) -> syn::Result<()> {
    let (mut outputs, mut markers, mut identities, mut selectors) = (
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
    );
    for slot in slots {
        if !outputs.insert(slot.output.value()) {
            return Err(duplicate(&slot.output, "slot"));
        }
        if !markers.insert(slot.marker.to_string()) {
            return Err(duplicate(&slot.output, "slot marker for"));
        }
        let selector = member_name(&slot.selector);
        if GENERATED.contains(&selector.as_str()) || !selectors.insert(selector.clone()) {
            return Err(syn::Error::new(
                slot.selector.span(),
                format!(
                    "selector `{selector}` is already declared for this projection; \
                     rename it with `selector = ..`"
                ),
            ));
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
