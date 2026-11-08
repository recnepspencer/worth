//! A generated input and its stable dependent-producer source are distinct records.
use super::*;
pub struct ReconciledSourceOf;
impl Relation for ReconciledSourceOf {
    const NAME: &'static str = "ReconciledSourceOf";
    type To = SecondOutput;
}
impl ReconciledSourceOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, SecondOutput> {
        relation(Storage, FirstOutput, Self)
    }
}
