//! One observed body-field condition shared by admission and read-only guidance.
use worth_foundational::facade::AspectValue;

pub enum BodyFieldExpectation<'a> {
    Any,
    Equals(&'a AspectValue),
    Present,
    /// The field (or its optional aspect) holds no value.
    Absent,
}

impl BodyFieldExpectation<'_> {
    pub fn matches(&self, observed: Option<&AspectValue>) -> bool {
        match self {
            Self::Any => true,
            Self::Equals(expected) => observed == Some(*expected),
            Self::Present => observed.is_some(),
            Self::Absent => observed.is_none(),
        }
    }
}
