//! Moved whole from `domain_computation::primary_graph::application_attempt::observation`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationFieldObservation {
    Present(worth_foundational::facade::AspectValue),
    Absent,
    AspectAbsent,
    UnsupportedAspectShape,
}

impl WorthQueryApplicationFieldObservation {
    pub(in crate::domain_computation::primary_graph) fn into_value(
        self,
    ) -> Option<worth_foundational::facade::AspectValue> {
        match self {
            Self::Present(value) => Some(value),
            Self::Absent | Self::AspectAbsent | Self::UnsupportedAspectShape => None,
        }
    }
}

impl From<worth_relational::facade::runtime::RelationalProjectedFieldObservation<'_>>
    for WorthQueryApplicationFieldObservation
{
    fn from(
        value: worth_relational::facade::runtime::RelationalProjectedFieldObservation<'_>,
    ) -> Self {
        use worth_relational::facade::runtime::RelationalProjectedFieldObservation as Raw;
        match value {
            Raw::Present(value) => Self::Present(value.clone()),
            Raw::AspectAbsent => Self::AspectAbsent,
            Raw::FieldAbsent => Self::Absent,
            Raw::UnsupportedAspectShape => Self::UnsupportedAspectShape,
        }
    }
}
