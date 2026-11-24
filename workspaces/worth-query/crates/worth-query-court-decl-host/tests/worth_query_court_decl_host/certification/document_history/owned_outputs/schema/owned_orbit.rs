//! Generated reconciliation members carry their own complete incidence.
use super::*;

pub struct Incidence;
impl Entity for Incidence {
    const NAME: &'static str = "Incidence";
    type Aspects = (IncidenceFacts,);
    type Relations = (OwnedNext, Beside);
}
impl Incidence {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
pub struct IncidenceFacts;
impl Aspect for IncidenceFacts {
    const NAME: &'static str = "IncidenceFacts";
    type Row = IncidenceRow;
}
pub struct IncidenceRow {
    pub key: String,
}
impl Record<IncidenceRow> for Bindings {
    fn record() -> impl Shape<IncidenceRow, Self> {
        field("IncidenceKey", |row: &IncidenceRow| row.key.clone())
            .equality_queryable(true)
            .unique()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IncidenceRow { key })
            })
    }
}
pub type IncidenceKey = FieldSelection<Storage, Incidence, IncidenceFacts, String>;

pub struct OwnedNext;
impl Relation for OwnedNext {
    const NAME: &'static str = "OwnedNext";
    type To = Incidence;
    const ORBIT: bool = true;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl OwnedNext {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Incidence, Incidence> {
        relation(Storage, Incidence, Self)
    }
}
/// A plain relation between the same records, under no orbit law: one
/// producer can link it on the records of another's ring.
pub struct Beside;
impl Relation for Beside {
    const NAME: &'static str = "Beside";
    type To = Incidence;
}
impl Beside {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Incidence, Incidence> {
        relation(Storage, Incidence, Self)
    }
}
