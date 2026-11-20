//! Port implementations bridging `worth-query-execution` primary-graph types to
//! `worth-query-execution-runtime` lower ports.

use std::any::Any;
use std::sync::{Arc, Mutex};

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableRecoveryClaimIdentity, ManagedRunObligationIdentity,
    RecoveryClaimDisposition,
};
use worth_query_execution_contracts::facade::{
    HeadCellRegistrationStop, QueryDispatchJournalDenial, QueryRecordedRecoveryClaim,
};
use worth_query_execution_obligations::facade::query_obligations::{
    QueryManagedCleanupOriginal, QueryManagedRunPosture, QueryManagedRunRecovery,
    QueryObligationDenial,
};
use worth_query_execution_runtime::facade::basis::WorthQueryProductBranchLease;
use worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime;
use worth_query_execution_runtime::facade::ports::{
    BranchCommitCoordinationPort, ForkLineagePort, InspectionTargetPort, ManagedRunObligationPort,
    PrerequisiteDenialPort, PrimaryMutationLease, PrimaryReadCustody, ProductRuntimeRef,
    RecoveryDispatchJournalPort, RelationalSourceInvalidationPort, RuntimeHomeRetirementDenial,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationInspectionContract,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::mvcc::RelationalPublicationCompanion;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription;
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchObservation};

use crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::primary_graph::output_lineage::{
    SourceInvalidationOwner, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::provider::{
    RegisteredPrimaryRead, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

impl RelationalSourceInvalidationPort for SourceInvalidationOwner {
    fn mint_cell_at_head(
        &self,
        subscription: &RelationalBridgeCanonicalSubscription,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
    ) -> Result<(), HeadCellRegistrationStop> {
        SourceInvalidationOwner::mint_cell_at_head(self, subscription, runtime, branch)
    }

    fn as_publication_companion(self: Arc<Self>) -> Arc<dyn RelationalPublicationCompanion> {
        self
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl PrimaryReadCustody for RegisteredPrimaryRead {
    fn grouped_count(&self) -> usize {
        self.facts().grouped_count()
    }

    fn contains_key(&self, identity: &str) -> bool {
        self.facts().contains_key(identity)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

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
    ) -> Result<Option<(ManagedRunObligationIdentity, Option<QueryObligationDenial>)>, QueryObligationDenial>
    {
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

impl RecoveryDispatchJournalPort for WorthQueryDispatchJournal {
    fn dispose_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
        identity: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        self.begin_recovery_admission(request)?
            .dispose(original, identity, terminal, request)
    }

    fn recorded_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Result<Option<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial> {
        WorthQueryDispatchJournal::recorded_recovery_claim(self, original)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl InspectionTargetPort for WorthQueryApplicationInspectionTarget {
    fn contract(&self) -> &WorthQueryInstalledApplicationInspectionContract {
        WorthQueryApplicationInspectionTarget::contract(self)
    }

    fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        WorthQueryApplicationInspectionTarget::schema_binding(self)
    }

    fn runtime_authority(&self) -> u64 {
        WorthQueryApplicationInspectionTarget::runtime_authority(self)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl<Schema: ApplicationSchema> ProductRuntimeRef
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_runtime(&self) -> &WorthQueryProductRuntime {
        &self.runtime.product_runtime.0
    }
}

pub(crate) struct OutputLineageForkAdapter(
    pub(crate) Arc<Mutex<WorthQueryApplicationOutputLineage>>,
);

impl ForkLineagePort for OutputLineageForkAdapter {
    fn register_fork(
        &self,
        source: &ProductBranchObservation,
        destination: &ProductBranchObservation,
    ) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .register_fork(source, destination);
    }
}

#[derive(Clone)]
pub(crate) struct PrimaryGraphProviderArc(pub(crate) Arc<WorthQueryPrimaryGraphProvider>);

impl std::ops::Deref for PrimaryGraphProviderArc {
    type Target = WorthQueryPrimaryGraphProvider;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for PrimaryGraphProviderArc {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for PrimaryGraphProviderArc {}

impl BranchCommitCoordinationPort for WorthQueryPrimaryGraphProvider {
    fn with_occurrence_coordination(
        &self,
        occurrence: ProductBranchIncarnation,
        body: &mut dyn FnMut(),
    ) {
        let lane = self.application_branch_commit_lane_for_occurrence(occurrence);
        let _guard = lane.enter();
        body();
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

impl From<RuntimeHomeRetirementDenial> for WorthQueryHomeRetirementDenial {
    fn from(value: RuntimeHomeRetirementDenial) -> Self {
        match value {
            RuntimeHomeRetirementDenial::Busy(owner) => Self::Busy(owner),
            RuntimeHomeRetirementDenial::Unsupported(owner) => Self::Unsupported(owner),
            RuntimeHomeRetirementDenial::NativeAlias(cause) => Self::NativeAlias(cause),
            RuntimeHomeRetirementDenial::NativeObservation(cause) => Self::NativeObservation(cause),
            RuntimeHomeRetirementDenial::BridgeBasis(cause) => Self::BridgeBasis(cause),
            RuntimeHomeRetirementDenial::CanonicalSubscription(cause) => {
                Self::CanonicalSubscription(cause)
            }
            RuntimeHomeRetirementDenial::Bridge(cause) => Self::Bridge(cause),
            RuntimeHomeRetirementDenial::Publication(cause) => Self::Publication(cause),
            RuntimeHomeRetirementDenial::Execution(cause) => Self::Execution(cause),
        }
    }
}
