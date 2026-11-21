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

impl SourceReadSupportPort for SourceReadSupport {
    fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.belongs_to_session(session)
    }

    fn validate_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop,
    > {
        use crate::domain_computation::primary_graph::source_facts::map_source_authorization_currentness;
        self.validate_current(execution, runtime, snapshot)
            .map_err(map_source_authorization_currentness)
    }

    fn same_conventional_support_dyn(
        &self,
        other: &dyn SourceReadSupportPort,
        admit: &mut dyn FnMut(u64) -> Result<(), Box<dyn Any>>,
    ) -> Result<bool, Box<dyn Any>> {
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return Ok(false);
        };
        self.same_conventional_support(other, &mut |units| admit(units))
    }

    fn visit_loci_dyn(
        &self,
        visit: &mut dyn FnMut(SourceAuthorizationLocus<'_>) -> Result<(), Box<dyn Any>>,
    ) -> Result<bool, Box<dyn Any>> {
        self.visit_loci(&mut |locus| visit(locus))
    }

    fn principal_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.principal_facts()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
