//! Moved whole from `domain_computation::primary_graph::application_query::observed_source`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQuerySourceExpectationKind {
    Row,
    ResultSet,
}

impl WorthQuerySourceExpectationKind {
    pub(in crate::domain_computation) const fn code(self) -> u64 {
        match self {
            Self::Row => 1,
            Self::ResultSet => 2,
        }
    }
}
