use super::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    RetainedSourceFactValues, RetainedSourceFacts,
};

/// Ordinary admitted facts keep their original order. Projected decision and
/// producer-source facts retain separate canonical maps until completion.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFacts {
    Admitted(Vec<WorthQueryApplicationObservedFact>),
    Projected(Option<RetainedSourceFacts>, Option<RetainedSourceFacts>),
}
impl SourceFacts {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn len(
        &self,
    ) -> usize {
        match self {
            Self::Admitted(facts) => facts.len(),
            Self::Projected(handler, source) => {
                handler.as_ref().map_or(0, RetainedSourceFacts::len)
                    + source.as_ref().map_or(0, RetainedSourceFacts::len)
            }
        }
    }
    /// Every dependent read precedes the producer's prepared-source suffix.
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn handler_len(
        &self,
    ) -> usize {
        match self {
            Self::Admitted(_) => 0,
            Self::Projected(handler, _) => handler.as_ref().map_or(0, RetainedSourceFacts::len),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn into_values(
        self,
    ) -> SourceFactValues {
        match self {
            Self::Admitted(facts) => SourceFactValues::Admitted(facts.into_iter()),
            Self::Projected(handler, source) => SourceFactValues::Projected(
                handler.map(RetainedSourceFacts::into_values),
                source.map(RetainedSourceFacts::into_values),
            ),
        }
    }
}
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) enum SourceFactValues
{
    Admitted(std::vec::IntoIter<WorthQueryApplicationObservedFact>),
    Projected(
        Option<RetainedSourceFactValues>,
        Option<RetainedSourceFactValues>,
    ),
}
impl Iterator for SourceFactValues {
    type Item = WorthQueryApplicationObservedFact;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Admitted(facts) => facts.next(),
            Self::Projected(handler, source) => handler
                .as_mut()
                .and_then(Iterator::next)
                .or_else(|| source.as_mut().and_then(Iterator::next)),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Admitted(facts) => facts.size_hint(),
            Self::Projected(handler, source) => {
                let len = handler.as_ref().map_or(0, ExactSizeIterator::len)
                    + source.as_ref().map_or(0, ExactSizeIterator::len);
                (len, Some(len))
            }
        }
    }
}
impl ExactSizeIterator for SourceFactValues {}
