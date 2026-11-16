//! A declared parent of a pending child is distinct from an outgoing consumer.
use super::*;
pub struct PendingChildOf;
impl Relation for PendingChildOf {
    const NAME: &'static str = "PendingChildOf";
    type To = FirstOutput;
}
impl PendingChildOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, FirstOutput> {
        relation(Storage, Member, Self)
    }
}
