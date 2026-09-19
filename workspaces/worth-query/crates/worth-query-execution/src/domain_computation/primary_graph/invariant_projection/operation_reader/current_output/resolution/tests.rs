use super::{classify_current_entities, CurrentOutputCardinality};

#[test]
fn current_role_cardinality_ignores_repeated_identity_but_not_distinct_outputs() {
    assert_eq!(
        classify_current_entities(Vec::<u64>::new()),
        CurrentOutputCardinality::Missing
    );
    assert_eq!(
        classify_current_entities([11, 11]),
        CurrentOutputCardinality::Unique(11)
    );
    assert_eq!(
        classify_current_entities([11, 11, 12, 12]),
        CurrentOutputCardinality::Ambiguous(vec![11, 12])
    );
}
