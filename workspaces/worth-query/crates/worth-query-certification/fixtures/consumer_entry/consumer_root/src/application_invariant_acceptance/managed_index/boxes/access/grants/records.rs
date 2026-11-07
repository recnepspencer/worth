// One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct BoxGrantRecords;
impl fragment::Fragment for BoxGrantRecords {
    const NAME: &'static str = "box_grant_records";
    type Bindings = BoxGrantRecordsBindings;
    type Members = (BoxGrant,);
}
pub struct BoxGrantRecordsBindings;
impl fragment::BindingSet for BoxGrantRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for BoxGrantRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<u64> for BoxGrantRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct BoxGrant;
impl fragment::Entity for BoxGrant {
    const NAME: &'static str = "BoxGrant";
    type Aspects = (BoxGrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl BoxGrant {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(BoxGrantRecords, Self)
    }
}
pub struct BoxGrantFacts;
impl fragment::Aspect for BoxGrantFacts {
    const NAME: &'static str = "BoxGrantFacts";
    type Row = BoxGrantFactsValue;
}

pub struct BoxGrantFactsValue {
    pub grant_identity: String,
    pub grant_action: u64,
    pub grant_purpose: u64,
    pub grant_status: u64,
    pub grant_read_status: u64,
    pub grant_scope: String,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_depth: u64,
    pub grant_disclosure: u64,
}
impl fragment::Record<BoxGrantFactsValue> for BoxGrantRecordsBindings {
    fn record() -> impl fragment::Shape<BoxGrantFactsValue, Self> {
        fragment::field("GrantIdentity", |row: &BoxGrantFactsValue| {
            row.grant_identity.clone()
        }).equality_queryable(true)
        .and("GrantAction", |row: &BoxGrantFactsValue| row.grant_action).equality_queryable(true)
        .read_only()
        .and("GrantPurpose", |row: &BoxGrantFactsValue| row.grant_purpose).equality_queryable(true)
        .read_only()
        .and("GrantStatus", |row: &BoxGrantFactsValue| row.grant_status).equality_queryable(true)
        .and("GrantReadStatus", |row: &BoxGrantFactsValue| {
            row.grant_read_status
        }).equality_queryable(true)
        .and("GrantScope", |row: &BoxGrantFactsValue| {
            row.grant_scope.clone()
        }).equality_queryable(true)
        .and("GrantNotBefore", |row: &BoxGrantFactsValue| {
            row.grant_not_before
        }).equality_queryable(true)
        .read_only()
        .and("GrantNotAfter", |row: &BoxGrantFactsValue| {
            row.grant_not_after
        }).equality_queryable(true)
        .read_only()
        .and("GrantDepth", |row: &BoxGrantFactsValue| row.grant_depth).equality_queryable(true)
        .read_only()
        .and("GrantDisclosure", |row: &BoxGrantFactsValue| {
            row.grant_disclosure
        }).equality_queryable(true)
        .read_only()
        .build(
            |grant_identity,
             grant_action,
             grant_purpose,
             grant_status,
             grant_read_status,
             grant_scope,
             grant_not_before,
             grant_not_after,
             grant_depth,
             grant_disclosure,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(BoxGrantFactsValue {
                    grant_identity,
                    grant_action,
                    grant_purpose,
                    grant_status,
                    grant_read_status,
                    grant_scope,
                    grant_not_before,
                    grant_not_after,
                    grant_depth,
                    grant_disclosure,
                })
            },
        )
    }
}
pub type GrantIdentity = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, String>;
pub type GrantAction = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantPurpose = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantStatus = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantReadStatus = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantScope = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, String>;
pub type GrantNotBefore = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantDepth = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub type GrantDisclosure = FieldSelection<BoxGrantRecords, BoxGrant, BoxGrantFacts, u64>;
pub struct GrantResource;
impl fragment::Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Directory;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, BoxGrant, Directory> {
        relation(BoxGrantRecords, BoxGrant, Self)
    }
}
pub struct GrantGrantee;
impl fragment::Relation for GrantGrantee {
    const NAME: &'static str = "GrantGrantee";
    type To = BoxGrant;
}
impl GrantGrantee {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Principal, BoxGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            <Self as Relation>::NAME,
            <Principal as Entity>::NAME,
            <BoxGrant as Entity>::NAME,
            relation_integrity::<Self>(),
        )
    }
}
pub struct GrantGrantor;
impl fragment::Relation for GrantGrantor {
    const NAME: &'static str = "GrantGrantor";
    type To = BoxGrant;
}
impl GrantGrantor {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Principal, BoxGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            <Self as Relation>::NAME,
            <Principal as Entity>::NAME,
            <BoxGrant as Entity>::NAME,
            relation_integrity::<Self>(),
        )
    }
}
pub struct GrantParent;
impl fragment::Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = BoxGrant;
}
impl GrantParent {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, BoxGrant, BoxGrant> {
        relation(BoxGrantRecords, BoxGrant, Self)
    }
}
