//! Graph history qualification preserves its own control and completeness causes.
#[derive(Debug)]
pub enum WorthQueryHistoryOutputQualificationDenial {
    Source(Box<crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial>),
    ParentMismatch {
        source: worth_relational::facade::identity::EntityId,
        family: &'static str,
    },
    ParentEvidenceUnavailable {
        source: worth_relational::facade::identity::EntityId,
        family: &'static str,
    },
    Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial),
    Control(Box<super::super::WorthQueryHistoryNavigationDenial>),
    /// The authorized model's selected routes were not all verified. No child
    /// source identity is disclosed without that source's current admission.
    Incomplete {
        model: worth_relational::facade::identity::EntityId,
    },
    SelectionRead(worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial),
}
impl From<crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial>
    for WorthQueryHistoryOutputQualificationDenial
{
    fn from(
        cause: crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial,
    ) -> Self {
        Self::Original(cause)
    }
}
