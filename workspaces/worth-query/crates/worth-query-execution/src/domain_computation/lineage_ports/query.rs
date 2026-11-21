//! Authorization-support, governance, and diagnostic-admission port implementations.

use std::any::Any;

use worth_query_execution_lineage::facade::ports::{
    DiagnosticComparisonAdmissionPort, QueryGovernancePort, SourceAuthorizationLocus,
    SourceReadSupportPort,
};

use crate::domain_computation::authorization::application_disclosure::decision::{
    DiagnosticComparisonAdmission, WorthQueryApplicationQueryGovernance,
};
use crate::domain_computation::authorization::SourceReadSupport;

impl DiagnosticComparisonAdmissionPort for DiagnosticComparisonAdmission<'_> {
    fn graph(&self) -> &worth_query_installation::facade::WorthQueryInstalledGraphReadContract {
        self.graph()
    }
}

impl QueryGovernancePort for WorthQueryApplicationQueryGovernance {
    fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }

    fn is_disclosed(
        &self,
        slot: &worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
    ) -> bool {
        self.is_disclosed(slot)
    }

    fn omission(
        &self,
        slot: &worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
    ) -> Option<(&str, &worth_foundational::facade::AspectValue)> {
        self.omission(slot)
    }
}


