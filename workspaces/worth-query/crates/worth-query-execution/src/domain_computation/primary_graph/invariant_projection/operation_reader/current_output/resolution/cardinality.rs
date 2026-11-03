use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum CurrentOutputCardinality<T> {
    Missing,
    Unique(T),
    Ambiguous(Vec<T>),
}

pub(super) fn classify_current_entities<T: Copy + Eq + Hash>(
    entities: impl IntoIterator<Item = T>,
) -> CurrentOutputCardinality<T> {
    let mut seen = HashSet::new();
    let mut unique = entities
        .into_iter()
        .filter(|entity| seen.insert(*entity))
        .collect::<Vec<_>>();
    match unique.len() {
        0 => CurrentOutputCardinality::Missing,
        1 => CurrentOutputCardinality::Unique(unique.pop().expect("one unique entity")),
        _ => CurrentOutputCardinality::Ambiguous(unique),
    }
}
