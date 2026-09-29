//! The temporal idempotency relation is derived from every part of the intent,
//! never spelled by hand.

use worth_query_installation::facade::WorthQueryTemporalIntentIdempotencyRelation as Relation;

fn relation(identity: &str, revision: u64, input: &str) -> Relation {
    Relation::declare(&(identity, revision, input)).expect("the parts encode")
}

#[test]
fn equal_parts_derive_equal_relations_of_a_fixed_shape() {
    let relation = relation("intent-1", 3, "wake");
    assert_eq!(relation, self::relation("intent-1", 3, "wake"));
    assert_eq!(relation.as_str().len(), 64);
    assert!(relation
        .as_str()
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn a_delimiter_moved_between_parts_is_a_different_relation() {
    assert_ne!(relation("a:1", 2, "x"), relation("a", 2, "1:x"));
    assert_ne!(relation("ab", 2, "c"), relation("a", 2, "bc"));
}

#[test]
fn a_changed_revision_or_input_is_a_different_relation() {
    let baseline = relation("intent-1", 3, "wake");
    assert_ne!(baseline, relation("intent-1", 4, "wake"));
    assert_ne!(baseline, relation("intent-1", 3, "wake-again"));
    assert_ne!(baseline, relation("intent-2", 3, "wake"));
}
