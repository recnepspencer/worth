//! One observed body-field condition shared by admission and read-only guidance.
use worth_foundational::facade::AspectValue;

pub(in crate::domain_computation::primary_graph) enum BodyFieldExpectation<'a> {
    Any,
    Equals(&'a AspectValue),
    Present,
}

impl BodyFieldExpectation<'_> {
    pub(in crate::domain_computation::primary_graph) fn matches(
        &self,
        observed: Option<&AspectValue>,
    ) -> bool {
        match self {
            Self::Any => true,
            Self::Equals(expected) => observed == Some(*expected),
            Self::Present => observed.is_some(),
        }
    }
}
