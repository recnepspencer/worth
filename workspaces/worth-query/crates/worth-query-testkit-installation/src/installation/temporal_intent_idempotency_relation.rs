//! The temporal idempotency relation is derived from every part of the intent,
//! never spelled by hand.

pub use worth_query_installation::facade::WorthQueryTemporalIntentIdempotencyRelation as Relation;

pub fn relation(identity: &str, revision: u64, input: &str) -> Relation {
    Relation::declare(&(identity, revision, input)).expect("the parts encode")
}
