//! Crate-boundary port for internal `WorthQueryAdvancementPhase` accessors.
//! Never exported on `worth-query-execution::facade` or Host.

use worth_execution::ExecutionRequest;

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    QueryRequestExecution, TestHostRequest, WorthQueryAdvancementPhase,
    WorthQueryForeignAdvancementPhase,
};

pub trait AdvancementPhaseInternalPort<'scope> {
    fn world_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>;

    fn request_for_owner(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase>;

    fn request_for_source(
        &self,
        source_instance: u64,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase>;

    fn execution_request_for(
        &self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase>;

    fn execution_for(
        &self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase>;

    fn test_host_request(&self) -> TestHostRequest<'scope>;

    /// The advancement's single trusted admission instant; `first` runs only
    /// when nothing in this advancement has fixed it yet.
    fn settle_admitted_at(&self, first: impl FnOnce() -> AdmittedAt) -> AdmittedAt;
}

type AdmittedAt = Result<
    worth_foundational::facade::CanonicalTimestamp,
    worth_query_execution_contracts::facade::runtime_time::WorthQueryRuntimeTimeDenial,
>;

impl<'scope> AdvancementPhaseInternalPort<'scope> for WorthQueryAdvancementPhase<'scope> {
    fn world_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity> {
        Self::world_owner(self)
    }

    fn request_for_owner(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        Self::request_for_owner(self, owner)
    }

    fn request_for_source(
        &self,
        source_instance: u64,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        Self::request_for_source(self, source_instance)
    }

    fn execution_request_for(
        &self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        Self::execution_request_for(self, runtime)
    }

    fn execution_for(
        &self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase> {
        Self::execution_for(self, runtime)
    }

    fn test_host_request(&self) -> TestHostRequest<'scope> {
        Self::test_host_request(self)
    }

    fn settle_admitted_at(&self, first: impl FnOnce() -> AdmittedAt) -> AdmittedAt {
        Self::settle_admitted_at(self, first)
    }
}
