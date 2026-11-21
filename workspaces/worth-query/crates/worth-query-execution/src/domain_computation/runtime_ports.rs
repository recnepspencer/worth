//! Port implementations bridging `worth-query-execution` primary-graph types to
//! `worth-query-execution-runtime` lower ports.

use std::any::Any;
use std::sync::{Arc, Mutex};

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableRecoveryClaimIdentity, RecoveryClaimDisposition,
};
use worth_query_execution_contracts::facade::{
    QueryDispatchJournalDenial, QueryRecordedRecoveryClaim,
};
use worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime;
use worth_query_execution_runtime::facade::ports::{
    BranchCommitCoordinationPort, ForkLineagePort, InspectionTargetPort, PrimaryReadCustody,
    ProductRuntimeRef, RecoveryDispatchJournalPort, RuntimeHomeRetirementDenial,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationInspectionContract,
};
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchObservation};

use crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal;
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::provider::{
    RegisteredPrimaryRead, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

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
