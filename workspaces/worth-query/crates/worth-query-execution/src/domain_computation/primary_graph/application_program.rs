use super::WorthQueryOutputDemandDenial;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::application_program::{
    WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryApplicationPrerequisiteOutputConnection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryPreparedRequiredOutputSource, WorthQueryRequiredOutputConnectionDenial,
};

/// A source publication succeeded, but its required-output recovery custody
/// could not be established.
pub struct WorthQueryRequiredOutputSourcePreparationFailure {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) denial: WorthQueryOutputDemandDenial,
    pub(in crate::domain_computation::primary_graph) source:
        super::application_installation::WorthQueryUnpublishedProgramOutputSource,
    pub(in crate::domain_computation::primary_graph) carrier:
        super::application_installation::WorthQueryRecoveredProgramOutputSource,
}

impl WorthQueryRequiredOutputSourcePreparationFailure {
    pub fn has_original_delivery(&self) -> bool {
        self.carrier.has_original_delivery()
    }

    pub fn receipt(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn denial(&self) -> &WorthQueryOutputDemandDenial {
        &self.denial
    }

    pub fn source(
        &self,
    ) -> &super::application_installation::WorthQueryUnpublishedProgramOutputSource {
        &self.source
    }

    pub fn returned(
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        denial: WorthQueryOutputDemandDenial,
        source: super::application_installation::WorthQueryUnpublishedProgramOutputSource,
        carrier: super::application_installation::WorthQueryRecoveredProgramOutputSource,
    ) -> Self {
        Self {
            receipt,
            denial,
            source,
            carrier,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        WorthQueryOutputDemandDenial,
        super::application_installation::WorthQueryUnpublishedProgramOutputSource,
        super::application_installation::WorthQueryRecoveredProgramOutputSource,
    ) {
        (self.receipt, self.denial, self.source, self.carrier)
    }
}

#[cfg(test)]
#[path = "application_program_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
