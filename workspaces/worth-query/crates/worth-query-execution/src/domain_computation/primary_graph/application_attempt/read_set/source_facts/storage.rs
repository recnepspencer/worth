use super::{WorthQueryApplicationFactStorageKey, WorthQueryApplicationObservedFact};
use std::collections::BTreeMap;

/// Ordinary admitted facts keep their original order. Projected dependencies
/// retain their canonical-keyed map until completion moves final values.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFacts {
    Admitted(Vec<WorthQueryApplicationObservedFact>),
    Projected(BTreeMap<WorthQueryApplicationFactStorageKey, WorthQueryApplicationObservedFact>),
}
impl SourceFacts {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn len(
        &self,
    ) -> usize {
        match self {
            Self::Admitted(facts) => facts.len(),
            Self::Projected(facts) => facts.len(),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn into_values(
        self,
    ) -> SourceFactValues {
        match self {
            Self::Admitted(facts) => SourceFactValues::Admitted(facts.into_iter()),
            Self::Projected(facts) => SourceFactValues::Projected(facts.into_values()),
        }
    }
}
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFactValues
{
    Admitted(std::vec::IntoIter<WorthQueryApplicationObservedFact>),
    Projected(
        std::collections::btree_map::IntoValues<
            WorthQueryApplicationFactStorageKey,
            WorthQueryApplicationObservedFact,
        >,
    ),
}
impl Iterator for SourceFactValues {
    type Item = WorthQueryApplicationObservedFact;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Admitted(facts) => facts.next(),
            Self::Projected(facts) => facts.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Admitted(facts) => facts.size_hint(),
            Self::Projected(facts) => facts.size_hint(),
        }
    }
}
impl ExactSizeIterator for SourceFactValues {}
