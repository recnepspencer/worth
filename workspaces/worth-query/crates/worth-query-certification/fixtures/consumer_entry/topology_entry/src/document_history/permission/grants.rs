//! Separate live navigation grants are outside the selected model record.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct HistoryGrantRecords;
impl Fragment for HistoryGrantRecords {
    const NAME: &'static str = "history_grants";
    type Bindings = HistoryGrantBindings;
    type Members = (PlanarHistoryGrant,);
}
pub struct HistoryGrantBindings;
impl BindingSet for HistoryGrantBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for HistoryGrantBindings {
    type Binding = StringApplicationValueBinding;
}
impl Scalar<u64> for HistoryGrantBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
pub struct PlanarHistoryGrant;
impl Entity for PlanarHistoryGrant {
    const NAME: &'static str = "PlanarHistoryGrant";
    type Aspects = (PlanarHistoryGrantFacts,);
    type Relations = (PlanarHistoryResource, PlanarHistoryParent);
}
impl PlanarHistoryGrant {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationEntityRef<S, Self> {
        entity(HistoryGrantRecords, Self)
    }
}
pub struct PlanarHistoryGrantFacts;
impl Aspect for PlanarHistoryGrantFacts {
    const NAME: &'static str = "PlanarHistoryGrantFacts";
    type Row = HistoryGrant;
}
pub struct HistoryGrant {
    pub value_0: String,
    pub value_1: String,
    pub value_2: String,
    pub value_3: String,
    pub value_4: String,
    pub value_5: u64,
    pub value_6: u64,
    pub value_7: u64,
}
impl fragment::Record<HistoryGrant> for HistoryGrantBindings {
    fn record() -> impl Shape<HistoryGrant, Self> {
        field("PlanarHistoryGrantIdentity", |row: &HistoryGrant| {
            row.value_0.clone()
        })
        .equality_queryable(true)
        .unique()
        .and("PlanarHistoryScopeKey", |row: &HistoryGrant| {
            row.value_1.clone()
        })
        .equality_queryable(true)
        .and("PlanarHistoryAction", |row: &HistoryGrant| {
            row.value_2.clone()
        })
        .equality_queryable(true)
        .and("PlanarHistoryPurpose", |row: &HistoryGrant| {
            row.value_3.clone()
        })
        .equality_queryable(true)
        .and("PlanarHistoryStatus", |row: &HistoryGrant| {
            row.value_4.clone()
        })
        .equality_queryable(true)
        .and("PlanarHistoryNotBefore", |row: &HistoryGrant| row.value_5)
        .equality_queryable(true)
        .and("PlanarHistoryNotAfter", |row: &HistoryGrant| row.value_6)
        .equality_queryable(true)
        .and("PlanarHistoryDepth", |row: &HistoryGrant| row.value_7)
        .equality_queryable(true)
        .build(
            |value_0,
             value_1,
             value_2,
             value_3,
             value_4,
             value_5,
             value_6,
             value_7,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(HistoryGrant {
                    value_0,
                    value_1,
                    value_2,
                    value_3,
                    value_4,
                    value_5,
                    value_6,
                    value_7,
                })
            },
        )
    }
}
pub type PlanarHistoryGrantIdentity =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, String>;
pub type PlanarHistoryScopeKey =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, String>;
pub type PlanarHistoryAction =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, String>;
pub type PlanarHistoryPurpose =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, String>;
pub type PlanarHistoryStatus =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, String>;
pub type PlanarHistoryNotBefore =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, u64>;
pub type PlanarHistoryNotAfter =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, u64>;
pub type PlanarHistoryDepth =
    FieldSelection<HistoryGrantRecords, PlanarHistoryGrant, PlanarHistoryGrantFacts, u64>;
pub struct PlanarHistoryResource;
impl Relation for PlanarHistoryResource {
    const NAME: &'static str = "PlanarHistoryResource";
    type To = Body;
}
impl PlanarHistoryResource {
    pub fn reference<S: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, PlanarHistoryGrant, Body> {
        relation(HistoryGrantRecords, PlanarHistoryGrant, Self)
    }
}
pub struct PlanarHistoryParent;
impl Relation for PlanarHistoryParent {
    const NAME: &'static str = "PlanarHistoryParent";
    type To = PlanarHistoryGrant;
}
impl PlanarHistoryParent {
    pub fn reference<S: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, PlanarHistoryGrant, PlanarHistoryGrant> {
        relation(HistoryGrantRecords, PlanarHistoryGrant, Self)
    }
}
pub struct PlanarHistoryGrantor;
impl Relation for PlanarHistoryGrantor {
    const NAME: &'static str = "PlanarHistoryGrantor";
    type To = PlanarHistoryGrant;
}
impl PlanarHistoryGrantor {
    pub fn reference<S: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, Principal, PlanarHistoryGrant> {
        relation(PrincipalRecords, Principal, Self)
    }
}
pub struct PlanarHistoryGrantee;
impl Relation for PlanarHistoryGrantee {
    const NAME: &'static str = "PlanarHistoryGrantee";
    type To = PlanarHistoryGrant;
}
impl PlanarHistoryGrantee {
    pub fn reference<S: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, Principal, PlanarHistoryGrant> {
        relation(PrincipalRecords, Principal, Self)
    }
}
