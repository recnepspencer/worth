//! An independent scalar cohort used only by the unindexed-support fixture.
use super::*;
pub struct Cap;
impl Entity for Cap {
    const NAME: &'static str = "Cap";
    type Aspects = (CapFacts,);
    type Relations = (CapNext,);
}
impl Cap {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
pub struct CapFacts;
impl Aspect for CapFacts {
    const NAME: &'static str = "CapFacts";
    type Row = CapRow;
}
pub struct CapRow {
    pub key: String,
    pub value: u64,
}
impl Record<CapRow> for Bindings {
    fn record() -> impl Shape<CapRow, Self> {
        field("CapKey", |row: &CapRow| row.key.clone())
            .equality_queryable(true)
            .unique()
            .and("CapValue", |row: &CapRow| row.value)
            .equality_queryable(true)
            .build(|key, value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapRow { key, value })
            })
    }
}
pub type CapKey = FieldSelection<Storage, Cap, CapFacts, String>;
pub type CapValue = FieldSelection<Storage, Cap, CapFacts, u64>;
pub struct CapNext;
impl Relation for CapNext {
    const NAME: &'static str = "CapNext";
    type To = Cap;
    const ORBIT: bool = true;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl CapNext {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Cap, Cap> {
        relation(Storage, Cap, Self)
    }
}
