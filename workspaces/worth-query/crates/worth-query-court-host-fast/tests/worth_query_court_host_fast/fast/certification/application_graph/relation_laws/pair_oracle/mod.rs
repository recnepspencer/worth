pub(crate) use worth_query_worlds::facade::certification::relation_laws::pair_oracle::*;

#[test]
fn seeded_pairs_follow_independent_symmetry_meaning() {
    exercise::<Symmetric>(Law::Symmetric);
}

#[test]
fn seeded_pairs_follow_independent_directed_uniqueness_meaning() {
    exercise::<DirectedUnique>(Law::DirectedUnique);
}

#[test]
fn seeded_pairs_follow_independent_normalized_uniqueness_meaning() {
    exercise::<NormalizedUnique>(Law::NormalizedUnique);
}

#[test]
fn seeded_pairs_follow_independent_composed_meaning() {
    exercise::<SymmetricDirectedUnique>(Law::SymmetricDirectedUnique);
}
