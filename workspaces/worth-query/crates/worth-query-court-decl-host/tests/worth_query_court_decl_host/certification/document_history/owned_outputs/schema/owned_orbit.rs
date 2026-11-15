//! Generated reconciliation members carry their own complete incidence.
use super::*;

pub struct OwnedNext;
impl Relation for OwnedNext {
    const NAME: &'static str = "OwnedNext";
    type To = FirstOutput;
    const ORBIT: bool = true;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl OwnedNext {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, FirstOutput> {
        relation(Storage, FirstOutput, Self)
    }
}
