//! Lineage-layer port implementations for attempt-owned types.

use crate::domain_computation::authorization::SourceAuthorizationLocus;
use std::any::Any;
use worth_query_execution_lineage::facade::ports::SourceReadSupportPort;

pub mod aftermath;
pub mod attempt;
pub mod commit_authority;
pub mod lineage;
pub mod workflow_retention;

pub use aftermath::RetainedPreImageExt;
pub use commit_authority::CommitAuthorityBindingExt;
pub use workflow_retention::{PrimaryGraphHandleWorkflowExt, WorkflowRetentionStore};

impl worth_query_execution_lineage::facade::ports::SourceReadSupportPort
    for crate::domain_computation::authorization::SourceReadSupport
{
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
    >{
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
