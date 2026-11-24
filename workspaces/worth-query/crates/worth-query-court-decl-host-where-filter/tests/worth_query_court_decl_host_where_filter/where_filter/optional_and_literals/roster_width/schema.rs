//! Schema with one narrow (1-aspect) and one wide (8-aspect) entity sharing
//! an identical single-field residual `where_filter`.
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::*;
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Copy)]
pub struct RosterSchema;
impl Fragment for RosterSchema {
    const NAME: &'static str = "where_filter_roster_width";
    type Bindings = NoBindings;
    type Members = (RosterContribution,);
}
impl ApplicationSchema for RosterSchema {
    const OWNER: &'static str = "worth.query.where-filter.roster";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for RosterSchema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct RosterContribution;
impl ApplicationSchemaContribution<RosterSchema> for RosterContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("where-filter.roster.contribution.v1");
    type Members = (RosterBoard, RosterPrincipalBinding, RosterQueries);
}

#[derive(Clone, Copy)]
pub struct RosterBoard;
impl Fragment for RosterBoard {
    const NAME: &'static str = "roster";
    type Bindings = Bindings;
    type Members = (ExternalMapping, Principal, Group, NarrowItem, WideItem);
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}

macro_rules! entity {
    ($name:ident, $aspects:ty, $relations:ty) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $aspects;
            type Relations = $relations;
        }
    };
}
macro_rules! u64_aspect {
    ($name:ident, $row:ident, $field:literal, $eq:expr) => {
        pub struct $name;
        impl Aspect for $name {
            const NAME: &'static str = stringify!($name);
            type Row = $row;
        }
        pub struct $row {
            v: u64,
        }
        impl Record<$row> for Bindings {
            fn record() -> impl Shape<$row, Self> {
                let f = field($field, |row: &$row| row.v);
                if $eq {
                    f.equality_queryable(true).read_only()
                } else {
                    f
                }
                .build(|v, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>($row { v })
                })
            }
        }
    };
}
u64_aspect!(PrincipalFacts, PrincipalRow, "PrincipalIdentity", true);
u64_aspect!(GroupFacts, GroupRow, "GroupKey", true);
u64_aspect!(Pad1, Pad1Row, "Pad1", false);
u64_aspect!(Pad2, Pad2Row, "Pad2", false);
u64_aspect!(Pad3, Pad3Row, "Pad3", false);
u64_aspect!(Pad4, Pad4Row, "Pad4", false);
u64_aspect!(Pad5, Pad5Row, "Pad5", false);
u64_aspect!(Pad6, Pad6Row, "Pad6", false);
u64_aspect!(Pad7, Pad7Row, "Pad7", false);

entity!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
entity!(Principal, (PrincipalFacts,), ());
entity!(Group, (GroupFacts,), (HasNarrow, HasWide));
entity!(NarrowItem, (ItemCore,), ());
entity!(
    WideItem,
    (ItemCore, Pad1, Pad2, Pad3, Pad4, Pad5, Pad6, Pad7),
    ()
);

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    id: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for Bindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityRow| {
            row.id.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &ExternalIdentityRow| row.status)
        .equality_queryable(true)
        .build(|id, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { id, status })
        })
    }
}

pub struct ItemCore;
impl Aspect for ItemCore {
    const NAME: &'static str = "ItemCore";
    type Row = ItemCoreRow;
}
pub struct ItemCoreRow {
    key: u64,
    tag: String,
}
impl Record<ItemCoreRow> for Bindings {
    fn record() -> impl Shape<ItemCoreRow, Self> {
        field("ItemKey", |row: &ItemCoreRow| row.key)
            .equality_queryable(true)
            .read_only()
            .and("Tag", |row: &ItemCoreRow| row.tag.clone())
            .build(|key, tag, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ItemCoreRow { key, tag })
            })
    }
}

macro_rules! rel {
    ($name:ident, $to:ident) => {
        pub struct $name;
        impl Relation for $name {
            const NAME: &'static str = stringify!($name);
            type To = $to;
        }
    };
}
rel!(MappingTarget, Principal);
rel!(HasNarrow, NarrowItem);
rel!(HasWide, WideItem);

pub type ExtField = FieldSelection<
    RosterBoard,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type StatusField = FieldSelection<
    RosterBoard,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdField = FieldSelection<RosterBoard, Principal, PrincipalFacts, u64>;
pub type GroupKeyField = FieldSelection<RosterBoard, Group, GroupFacts, u64>;
pub type NarrowKey = FieldSelection<RosterBoard, NarrowItem, ItemCore, u64>;
pub type NarrowTag = FieldSelection<RosterBoard, NarrowItem, ItemCore, String>;
pub type WideKey = FieldSelection<RosterBoard, WideItem, ItemCore, u64>;
pub type WideTag = FieldSelection<RosterBoard, WideItem, ItemCore, String>;

pub struct RosterPrincipalBinding;
impl RosterPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        RosterSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExtField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    StatusField::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                    RosterBoard,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdField::at("PrincipalIdentity"),
                ),
            },
        )
    }
}
impl Registration<RosterSchema> for RosterPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RosterSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<RosterSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}

pub struct Params;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRow(pub u64);
worth_query_structured_value_binding!(pub ParamsBinding for Params { identity: "roster.params.v1" });
worth_query_structured_value_binding!(pub KeyRowBinding for KeyRow { identity: "roster.row.v1" });
pub struct KeySlot;
worth_query_portable_type!(KeySlot => "roster.key-slot.v1");

worth_query_application_query!(pub NarrowQuery for RosterSchema, identity "roster.narrow.v1",
    parameters ParamsBinding, result KeyRowBinding, scope Group => "Group", name "narrow_filter");
worth_query_application_query!(pub WideQuery for RosterSchema, identity "roster.wide.v1",
    parameters ParamsBinding, result KeyRowBinding, scope Group => "Group", name "wide_filter");

impl WorthQueryApplicationProjection<RosterSchema, NarrowQuery> for KeyRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, RosterSchema, NarrowQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(ApplicationQueryResultFieldRef::<
            NarrowQuery,
            KeySlot,
            RosterSchema,
            NarrowKey,
        >::new(
            "key", NarrowKey::at("ItemKey")
        ))?))
    }
}
impl WorthQueryApplicationProjection<RosterSchema, WideQuery> for KeyRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, RosterSchema, WideQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(ApplicationQueryResultFieldRef::<
            WideQuery,
            KeySlot,
            RosterSchema,
            WideKey,
        >::new(
            "key", WideKey::at("ItemKey")
        ))?))
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct RunNarrow {
    pub group: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct RunWide {
    pub group: u64,
}
worth_query_structured_value_binding!(pub RunNarrowInput for RunNarrow { identity: "roster.narrow-in.v1" });
worth_query_structured_value_binding!(pub RunWideInput for RunWide { identity: "roster.wide-in.v1" });

fn narrow_params(_: &RunNarrow) -> ApplicationQueryParameterSet<NarrowQuery> {
    ApplicationQueryParameterSet::new()
}
fn wide_params(_: &RunWide) -> ApplicationQueryParameterSet<WideQuery> {
    ApplicationQueryParameterSet::new()
}
fn narrow_group(input: RunNarrow) -> u64 {
    input.group
}
fn wide_group(input: RunWide) -> u64 {
    input.group
}

worth_query_query_binding!(pub NarrowBinding for RunNarrow, schema RosterSchema,
    identity "roster.narrow-binding.v1", input RunNarrowInput,
    query NarrowQuery, parameters ParamsBinding => |input: &RunNarrow| narrow_params(input),
    result KeyRowBinding, principal RosterPrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope GroupKeyField, field GroupKeyField::at("GroupKey"), value narrow_group, limits results 64);
worth_query_query_binding!(pub WideBinding for RunWide, schema RosterSchema,
    identity "roster.wide-binding.v1", input RunWideInput,
    query WideQuery, parameters ParamsBinding => |input: &RunWide| wide_params(input),
    result KeyRowBinding, principal RosterPrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope GroupKeyField, field GroupKeyField::at("GroupKey"), value wide_group, limits results 64);

macro_rules! item_query {
    ($query:ident, $item:ident, $rel:ident, $key:ident, $tag:ident) => {{
        let hit = ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
            "hit".to_owned(),
        )
        .expect("hit encodes");
        let key_ref = ApplicationQueryResultFieldRef::<$query, KeySlot, RosterSchema, $key>::new(
            "key",
            $key::at("ItemKey"),
        );
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("tag", $tag::at("Tag"))
            .and_then(|o| o.literal("hit", hit))
            .and_then(|o| o.parse("tag == hit"))
            .expect("filter parses");
        ApplicationQueryDefinitionBuilder::declare($query::reference())
            .root(entity(RosterBoard, $item))
            .scope(entity(RosterBoard, Group))
            .result_shape(
                ApplicationQueryResultShapeBuilder::<
                    RosterSchema,
                    $query,
                    $item,
                    KeyRow,
                    KeyRowBinding,
                >::new(entity(RosterBoard, $item))
                .field(key_ref)
                .build(),
            )
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
            .public()
            .root_path(
                ApplicationQueryRootPath::from(entity(RosterBoard, Group)).forward(relation(
                    RosterBoard,
                    Group,
                    $rel,
                )),
            )
            .where_filter(filter)
            .order_by(key_ref, ApplicationQueryOrderingDirection::Ascending)
    }};
}

pub struct RosterQueries;
impl Registration<RosterSchema> for RosterQueries {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RosterSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<RosterSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .application_query(item_query!(
                NarrowQuery,
                NarrowItem,
                HasNarrow,
                NarrowKey,
                NarrowTag
            ))
            .application_query_binding::<NarrowBinding>()
            .application_query(item_query!(WideQuery, WideItem, HasWide, WideKey, WideTag))
            .application_query_binding::<WideBinding>())
    }
}
