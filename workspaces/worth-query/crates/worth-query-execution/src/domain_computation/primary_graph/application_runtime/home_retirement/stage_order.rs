//! Stage ordering for primary-graph application home retirement.

use super::QueryRetirementStage;

impl QueryRetirementStage {
    pub(super) const fn next(&self) -> Self {
        match self {
            Self::Handlers => Self::InboundCustody,
            Self::InboundCustody => Self::TransportCustody,
            Self::TransportCustody => Self::Participation,
            Self::Participation => Self::InstalledConditionals,
            Self::InstalledConditionals => Self::RecoveredOutputs,
            Self::RecoveredOutputs => Self::Provider,
            Self::Provider => Self::Projection,
            Self::Projection => Self::Granular,
            Self::Granular => Self::ProductSource,
            Self::ProductSource => Self::ServingGraph,
            Self::ServingGraph => Self::Bridge,
            Self::Bridge => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }
}
