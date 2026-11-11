//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment;
pub struct CatalogRecords;
impl fragment::Fragment for CatalogRecords {
    const NAME: &'static str = "catalog_records";
    type Bindings = CatalogRecordsBindings;
    type Members = (ExternalMapping, Principal, Directory, Item);
}
pub struct CatalogRecordsBindings;
impl fragment::BindingSet for CatalogRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for CatalogRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for CatalogRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for CatalogRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for CatalogRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<CatalogName> for CatalogRecordsBindings {
    type Binding = NameBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<bool> for CatalogRecordsBindings {
    type Binding = BoolApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (
        super::boxes::access::grants::GrantGrantee,
        super::boxes::access::grants::GrantGrantor,
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(CatalogRecords, Self)
    }
}
pub struct Directory;
impl fragment::Entity for Directory {
    const NAME: &'static str = "Directory";
    type Aspects = (DirectoryFacts,);
    type Relations = (Contains,);
}
impl Directory {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(CatalogRecords, Self)
    }
}
pub struct Item;
impl fragment::Entity for Item {
    const NAME: &'static str = "Item";
    type Aspects = (ItemFacts,);
    type Relations = ();
}
impl Item {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(CatalogRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for CatalogRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}

pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalFactsValue> for CatalogRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct DirectoryFacts;
impl fragment::Aspect for DirectoryFacts {
    const NAME: &'static str = "DirectoryFacts";
    type Row = DirectoryFactsValue;
}

pub struct DirectoryFactsValue {
    pub directory_identity: String,
}
impl fragment::Record<DirectoryFactsValue> for CatalogRecordsBindings {
    fn record() -> impl fragment::Shape<DirectoryFactsValue, Self> {
        fragment::field("DirectoryIdentity", |row: &DirectoryFactsValue| {
            row.directory_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .build(|directory_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(DirectoryFactsValue { directory_identity })
        })
    }
}
pub struct ItemFacts;
impl fragment::Aspect for ItemFacts {
    const NAME: &'static str = "ItemFacts";
    type Row = ItemFactsValue;
}

pub struct ItemFactsValue {
    pub item_identity: String,
    pub item_name: CatalogName,
    pub item_enabled: bool,
    pub item_bounds:
        crate::application_invariant_acceptance::managed_index::boxes::coordinates::Coordinates,
}
impl fragment::Record<ItemFactsValue> for CatalogRecordsBindings {
    fn record() -> impl fragment::Shape<ItemFactsValue, Self> {
        fragment::field("ItemIdentity", |row: &ItemFactsValue| {
            row.item_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("ItemName", |row: &ItemFactsValue| row.item_name.clone())
        .equality_queryable(true)
        .and("ItemEnabled", |row: &ItemFactsValue| row.item_enabled)
        .equality_queryable(true)
        .and("ItemBounds", |row: &ItemFactsValue| row.item_bounds)
        .equality_queryable(true)
        .read_only()
        .build(
            |item_identity,
             item_name,
             item_enabled,
             item_bounds,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ItemFactsValue {
                    item_identity,
                    item_name,
                    item_enabled,
                    item_bounds,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    CatalogRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    CatalogRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<CatalogRecords, Principal, PrincipalFacts, u64>;
pub type DirectoryIdentity = FieldSelection<CatalogRecords, Directory, DirectoryFacts, String>;
pub type ItemIdentity = FieldSelection<CatalogRecords, Item, ItemFacts, String>;
pub type ItemName = FieldSelection<CatalogRecords, Item, ItemFacts, CatalogName>;
pub type ItemEnabled = FieldSelection<CatalogRecords, Item, ItemFacts, bool>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, ExternalMapping, Principal> {
        relation(CatalogRecords, ExternalMapping, Self)
    }
}
pub struct Contains;
impl fragment::Relation for Contains {
    const NAME: &'static str = "Contains";
    type To = Item;
}
impl Contains {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Directory, Item> {
        relation(CatalogRecords, Directory, Self)
    }
}

impl Scalar<crate::application_invariant_acceptance::managed_index::boxes::coordinates::Coordinates>
    for CatalogRecordsBindings
{
    type Binding =
        crate::application_invariant_acceptance::managed_index::boxes::coordinates::Binding;

    const WRITABLE: bool = false;
}
