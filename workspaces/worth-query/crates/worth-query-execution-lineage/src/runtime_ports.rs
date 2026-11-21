//! Port implementations bridging `worth-query-execution-lineage` types to
//! `worth-query-execution-runtime` lower ports.

use std::any::Any;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::ManagedRunObligationIdentity;
use worth_query_execution_obligations::facade::query_obligations::{
    QueryManagedCleanupOriginal, QueryManagedRunPosture, QueryManagedRunRecovery,
    QueryObligationDenial,
};
use worth_query_execution_runtime::facade::basis::WorthQueryProductBranchLease;
use worth_query_execution_runtime::facade::ports::{
    ManagedRunObligationPort, PrerequisiteDenialPort, PrimaryMutationLease,
};
use worth_relational::facade::snapshots::SnapshotHandle;

use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory;

impl PrimaryMutationLease for WorthQueryApplicationSnapshotLease {
    fn snapshot(&self) -> &SnapshotHandle {
        WorthQueryApplicationSnapshotLease::snapshot(self)
    }

    fn product(&self) -> &WorthQueryProductBranchLease {
        WorthQueryApplicationSnapshotLease::product(self)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

impl ManagedRunObligationPort for QueryObligationHistory {
    fn retain_managed_run(
        &self,
        request: &ExecutionRequest<'_, '_>,
        logical: &str,
        attempt: &str,
    ) -> Result<
        Option<(ManagedRunObligationIdentity, Option<QueryObligationDenial>)>,
        QueryObligationDenial,
    > {
        QueryObligationHistory::retain_managed_run(self, request, logical, attempt)
    }

    fn settle_managed_run_retention(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
    ) -> Result<bool, QueryObligationDenial> {
        QueryObligationHistory::settle_managed_run_retention(self, request, identity)
    }

    fn associate_managed_run_original(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        original: QueryManagedCleanupOriginal,
    ) -> Result<(), QueryObligationDenial> {
        QueryObligationHistory::associate_managed_run_original(self, request, identity, original)
    }

    fn associate_managed_run_recovery(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        cause: QueryManagedRunRecovery,
    ) -> Result<(), QueryObligationDenial> {
        QueryObligationHistory::associate_managed_run_recovery(self, request, identity, cause)
    }

    fn record_managed_run_handoff(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        posture: QueryManagedRunPosture,
        current_attempt: &str,
    ) -> Result<(), QueryObligationDenial> {
        QueryObligationHistory::record_managed_run_handoff(
            self,
            request,
            identity,
            posture,
            current_attempt,
        )
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl PrerequisiteDenialPort for WorthQueryOutputDemandDenial {
    fn clone_box(&self) -> Box<dyn PrerequisiteDenialPort> {
        Box::new(self.clone())
    }

    fn eq_port(&self, other: &dyn PrerequisiteDenialPort) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

impl PrerequisiteDenialPort for WorthQueryOutputDemandDenialKind {
    fn clone_box(&self) -> Box<dyn PrerequisiteDenialPort> {
        Box::new(self.clone())
    }

    fn eq_port(&self, other: &dyn PrerequisiteDenialPort) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}
