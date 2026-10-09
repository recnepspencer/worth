use super::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    RetainedSourceFactValues, RetainedSourceFacts,
};

/// Ordinary admitted facts keep their original order. Projected dependencies
/// retain their canonical-keyed map until completion moves final values.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFacts {
    Admitted(Vec<WorthQueryApplicationObservedFact>),
    Projected(Option<RetainedSourceFacts>),
}
impl SourceFacts {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn len(
        &self,
    ) -> usize {
        match self {
            Self::Admitted(facts) => facts.len(),
            Self::Projected(facts) => facts.as_ref().map_or(0, RetainedSourceFacts::len),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn into_values(
        self,
    ) -> SourceFactValues {
        match self {
            Self::Admitted(facts) => SourceFactValues::Admitted(facts.into_iter()),
            Self::Projected(facts) => {
                SourceFactValues::Projected(facts.map(RetainedSourceFacts::into_values))
            }
        }
    }
}
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFactValues
{
    Admitted(std::vec::IntoIter<WorthQueryApplicationObservedFact>),
    Projected(Option<RetainedSourceFactValues>),
}
impl Iterator for SourceFactValues {
    type Item = WorthQueryApplicationObservedFact;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Admitted(facts) => facts.next(),
            Self::Projected(facts) => facts.as_mut().and_then(Iterator::next),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Admitted(facts) => facts.size_hint(),
            Self::Projected(facts) => facts.as_ref().map_or((0, Some(0)), Iterator::size_hint),
        }
    }
}
impl ExactSizeIterator for SourceFactValues {}
