//! Three descendant levels use the same actual Children relation with distinct slots.
use super::{
    query::{self, ItemRow, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::{
    application_query::*, portable_identity::WorthQueryPortableType, *,
};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub coordinate: (u32, u64, u32),
    pub identity: String,
    pub name: String,
    pub badge: Option<bool>,
    pub detail: Option<bool>,
    pub children: Vec<Node>,
}
pub struct ChildRelation;
worth_query_portable_type!(ChildRelation => "nested-catalog.child-relation.v1");
pub struct ChildIdentity;
worth_query_portable_type!(ChildIdentity => "nested-catalog.child-identity.v1");
pub struct ChildBadge;
worth_query_portable_type!(ChildBadge => "nested-catalog.child-badge.v1");
pub struct ChildDetail;
worth_query_portable_type!(ChildDetail => "nested-catalog.child-detail.v1");
pub struct ChildName;
worth_query_portable_type!(ChildName => "nested-catalog.child-name.v1");
pub struct GrandRelation;
worth_query_portable_type!(GrandRelation => "nested-catalog.grand-relation.v1");
pub struct GrandIdentity;
worth_query_portable_type!(GrandIdentity => "nested-catalog.grand-identity.v1");
pub struct GrandName;
worth_query_portable_type!(GrandName => "nested-catalog.grand-name.v1");
pub struct LeafRelation;
worth_query_portable_type!(LeafRelation => "nested-catalog.leaf-relation.v1");
pub struct LeafIdentity;
worth_query_portable_type!(LeafIdentity => "nested-catalog.leaf-identity.v1");
pub struct LeafName;
worth_query_portable_type!(LeafName => "nested-catalog.leaf-name.v1");
pub struct ParentRelation;
worth_query_portable_type!(ParentRelation => "nested-catalog.parent-relation.v1");
pub struct ParentIdentity;
worth_query_portable_type!(ParentIdentity => "nested-catalog.parent-identity.v1");
pub struct ParentName;
worth_query_portable_type!(ParentName => "nested-catalog.parent-name.v1");
pub struct SiblingRelation;
worth_query_portable_type!(SiblingRelation => "nested-catalog.sibling-relation.v1");
pub struct SiblingIdentity;
worth_query_portable_type!(SiblingIdentity => "nested-catalog.sibling-identity.v1");
pub struct SiblingName;
worth_query_portable_type!(SiblingName => "nested-catalog.sibling-name.v1");
type Text<Q, Slot, Field> = ApplicationQueryResultFieldRef<Q, Slot, Catalog, Field>;
fn identity<Q, Slot: WorthQueryPortableType>() -> Text<Q, Slot, ItemIdentity> {
    ApplicationQueryResultFieldRef::new("identity", ItemIdentity::at("ItemIdentity"))
}
fn name<Q, Slot: WorthQueryPortableType>() -> Text<Q, Slot, ItemName> {
    ApplicationQueryResultFieldRef::new("name", ItemName::at("ItemName"))
}
fn relation<Q, Slot: WorthQueryPortableType>() -> ApplicationQueryResultRelationRef<
    Q,
    Slot,
    Catalog,
    Children,
    Item,
    Item,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", Children::reference())
}
type Builder<Q> = ApplicationQueryResultShapeBuilder<Catalog, Q, Item, ItemRow, ResultBinding>;
pub(super) fn shape<Q: ApplicationQueryMarkerIdentity<Catalog>>(
) -> TypedApplicationQueryResultShape<Catalog, Q, Item, ItemRow, ResultBinding> {
    Builder::new(Item::reference())
        .field(query::identity())
        .field(query::name())
        .field(query::enabled())
        .optional_aspect_field(query::badge::<Q, query::BadgeSlot>())
        .optional_field(query::detail::<Q, query::DetailSlot>())
        .relation(
            relation::<Q, ChildRelation>(),
            Builder::new(Item::reference())
                .field(identity::<Q, ChildIdentity>())
                .field(name::<Q, ChildName>())
                .optional_aspect_field(query::badge::<Q, ChildBadge>())
                .optional_field(query::detail::<Q, ChildDetail>())
                .relation(
                    relation::<Q, GrandRelation>(),
                    Builder::new(Item::reference())
                        .field(identity::<Q, GrandIdentity>())
                        .field(name::<Q, GrandName>())
                        .relation(
                            relation::<Q, LeafRelation>(),
                            Builder::new(Item::reference())
                                .field(identity::<Q, LeafIdentity>())
                                .field(name::<Q, LeafName>()),
                        ),
                ),
        )
        .relation(
            parents::<Q>(),
            Builder::new(Item::reference())
                .field(identity::<Q, ParentIdentity>())
                .field(name::<Q, ParentName>())
                .relation(
                    relation::<Q, SiblingRelation>(),
                    Builder::new(Item::reference())
                        .field(identity::<Q, SiblingIdentity>())
                        .field(name::<Q, SiblingName>()),
                ),
        )
        .build()
}
pub(super) fn project<Q: ApplicationQueryMarkerIdentity<Catalog>>(
    row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
) -> Result<Vec<Node>, WorthQueryApplicationProjectionDenial> {
    row.many(relation::<Q, ChildRelation>())?
        .iter()
        .map(|child| {
            let mut value = node::<Q, ChildIdentity, ChildName>(&child)?;
            value.badge = child.optional_aspect_field(query::badge::<Q, ChildBadge>())?;
            value.detail = child.optional_field(query::detail::<Q, ChildDetail>())?;
            value.children = child
                .many(relation::<Q, GrandRelation>())?
                .iter()
                .map(|grand| {
                    let mut value = node::<Q, GrandIdentity, GrandName>(&grand)?;
                    value.children = grand
                        .many(relation::<Q, LeafRelation>())?
                        .iter()
                        .map(|leaf| node::<Q, LeafIdentity, LeafName>(&leaf))
                        .collect::<Result<_, _>>()?;
                    Ok(value)
                })
                .collect::<Result<_, _>>()?;
            Ok(value)
        })
        .collect()
}
fn node<
    Q: ApplicationQueryMarkerIdentity<Catalog>,
    I: WorthQueryPortableType,
    N: WorthQueryPortableType,
>(
    row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
) -> Result<Node, WorthQueryApplicationProjectionDenial> {
    let id = row.entity_id();
    Ok(Node {
        coordinate: (
            id.partition_value(),
            id.local_slot_value(),
            id.generation_value(),
        ),
        identity: row.field(identity::<Q, I>())?,
        name: row.field(name::<Q, N>())?.into(),
        badge: None,
        detail: None,
        children: Vec::new(),
    })
}

fn parents<Q>() -> ApplicationQueryResultRelationRef<
    Q,
    ParentRelation,
    Catalog,
    Children,
    Item,
    Item,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("parents", Children::reference())
}
pub(super) fn project_parents<Q: ApplicationQueryMarkerIdentity<Catalog>>(
    row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
) -> Result<Vec<Node>, WorthQueryApplicationProjectionDenial> {
    row.many(parents::<Q>())?
        .iter()
        .map(|parent| {
            let mut value = node::<Q, ParentIdentity, ParentName>(&parent)?;
            value.children = parent
                .many(relation::<Q, SiblingRelation>())?
                .iter()
                .map(|sibling| node::<Q, SiblingIdentity, SiblingName>(&sibling))
                .collect::<Result<_, _>>()?;
            Ok(value)
        })
        .collect()
}
