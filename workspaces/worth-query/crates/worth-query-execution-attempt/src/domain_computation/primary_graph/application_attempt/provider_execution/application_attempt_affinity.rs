//! Exact owner-minted association for one application provider attempt.

use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::{history::BranchId, snapshots::SnapshotHandle};

use crate::domain_computation::{
    authorization::WorthQueryOperationAdmissionIdentity,
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    provider_session::{
        WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
        WorthQueryProviderSessionAffinity, WorthQueryProviderSessionAffinityIdentity,
        WorthQueryProviderSessionTerminalBinding, WorthQueryProviderSessionView,
    },
};

/// The pre-session axes owned by the admitted operation and pinned snapshot.
///
/// This value is move-only and private to the application-attempt state
/// machine. The provider session is joined exactly once by [`bind_session`].
pub struct WorthQueryApplicationAttemptBasis {
    pub core: WorthQueryApplicationAttemptCore,
    pub original_completion: Option<super::super::super::provider::OriginalCompletionReceiver>,
    pub product: crate::basis::WorthQueryProductBranchLease,
}

pub struct WorthQueryApplicationAttemptCore {
    pub completion: super::super::super::provider::OriginalCompletionPublisher,
    pub runtime: WorthQueryRuntimeAuthorityIdentity,
    pub operation_attempt: WorthQueryOperationAdmissionIdentity,
    pub installed_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub installed_operation: std::sync::Arc<str>,
    pub resource_binding: std::sync::Arc<str>,
    pub operation: std::sync::Arc<str>,
    pub snapshot: SnapshotHandle,
    pub graph_work_session: WorthQueryGraphWorkSessionIdentity,
    pub graph_work_managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub branch: BranchId,
    pub request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub admitted_request:
        crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness,
    pub operation_scope: crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
}

/// Inseparable authority association retained by the provider attempt store.
///
/// No descriptive identity or lower provider-session identity can construct
/// this value. It is minted only after the exact provider plan is checked
/// against the already-admitted operation, pinned snapshot, graph-work session,
/// and Relational branch.
pub struct WorthQueryApplicationAttemptAffinity {
    basis: WorthQueryApplicationAttemptCore,
    provider_session: WorthQueryProviderSessionTerminalBinding,
    product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryApplicationAttemptAffinityMismatch {
    Runtime,
    InstalledOperation,
    ResourceBinding,
    OperationAttempt,
    OperationSlot,
    SchemaBinding,
    Snapshot,
    GraphWorkSession,
    GraphWorkManagedRun,
    Product,
}

pub trait WorthQueryApplicationAttemptAffinityView {
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn installed_operation(&self) -> &str;
    fn resource_binding(&self) -> &str;
    fn operation_attempt(&self) -> Option<WorthQueryOperationAdmissionIdentity>;
    fn operation_slot(&self) -> Option<&str>;
    fn schema_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity>;
    fn snapshot(&self) -> Option<&SnapshotHandle>;
    fn graph_work_session(&self) -> Option<u64>;
    fn graph_work_managed_run(&self) -> Option<u64>;
}

impl WorthQueryApplicationAttemptAffinityView for WorthQueryProviderSessionTerminalBinding {
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.plan().runtime_authority()
    }
    fn installed_operation(&self) -> &str {
        self.plan().operation_identity()
    }
    fn resource_binding(&self) -> &str {
        self.plan().binding_identity()
    }
    fn operation_attempt(&self) -> Option<WorthQueryOperationAdmissionIdentity> {
        self.plan().application_operation_attempt()
    }
    fn operation_slot(&self) -> Option<&str> {
        self.plan().application_operation_slot()
    }
    fn schema_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        self.plan().application_schema_binding()
    }
    fn snapshot(&self) -> Option<&SnapshotHandle> {
        self.plan().application_snapshot()
    }
    fn graph_work_session(&self) -> Option<u64> {
        self.plan().graph_work_session_identity()
    }
    fn graph_work_managed_run(&self) -> Option<u64> {
        self.plan().graph_work_managed_run_identity()
    }
}

impl WorthQueryApplicationAttemptBasis {
    pub fn capture<Schema, Operation, Input, Scope>(
        application: &impl crate::ports::RuntimeAuthorityPort,
        admission: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        snapshot: &SnapshotHandle,
        product: &crate::basis::WorthQueryProductBranchLease,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, super::super::WorthQueryApplicationCommitOutcome>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        if admission.runtime_authority() != application.runtime_authority_identity()
            || snapshot.branch_id() != admission.graph_work_branch()
            || snapshot.branch_id()
                != product
                    .observation()
                    .basis()
                    .relational_basis()
                    .identity()
                    .branch_id()
            || snapshot.version_id()
                != product
                    .observation()
                    .basis()
                    .relational_basis()
                    .observation()
                    .version_id()
        {
            return Err(super::provider_denial::denied(
                super::super::WorthQueryApplicationCommitDenialStage::ManagedRunAdmission,
            ));
        }
        let (completion, receiver) =
            super::super::super::provider::OriginalCompletionPublisher::pending_slot(policy)
                .map_err(
                    super::super::provider_compare_denial::original_custody_allocation_denied,
                )?;
        Ok(Self {
            original_completion: Some(receiver),
            core: WorthQueryApplicationAttemptCore {
                completion,
                runtime: admission.runtime_authority(),
                operation_attempt: admission.admission_identity(),
                installed_binding: admission.binding_identity().clone(),
                installed_operation: admission.retain_installed_operation_fingerprint(),
                resource_binding: admission.retain_resource_binding_identity(),
                operation: admission.operation().into(),
                snapshot: snapshot.clone(),
                graph_work_session: admission.graph_work_session_identity(),
                graph_work_managed_run: admission.graph_work_managed_run_identity(),
                branch: admission.graph_work_branch().clone(),
                request: admission.publication_request().clone(),
                admitted_request: admission.retain_admitted_request(),
                operation_scope: admission.operation_scope_binding().clone(),
            },
            product: product.retained_clone(),
        })
    }

    pub fn take_original_completion(
        &mut self,
    ) -> super::super::super::provider::OriginalCompletionReceiver {
        self.original_completion
            .take()
            .expect("only the active original progression receives its result")
    }

    pub fn from_parts(
        core: WorthQueryApplicationAttemptCore,
        original_completion: Option<super::super::super::provider::OriginalCompletionReceiver>,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Self {
        Self {
            core,
            original_completion,
            product,
        }
    }

    pub fn retained_product(&self) -> crate::basis::WorthQueryProductBranchLease {
        self.product.retained_clone()
    }

    pub fn bind_live_session(
        self,
        provider_session: &WorthQueryProviderSessionAffinity<'_>,
    ) -> Result<WorthQueryApplicationAttemptAffinity, ()> {
        let provider_session = provider_session.terminal_binding();
        if !self.affinity_mismatches(&provider_session).is_empty() {
            return Err(());
        }
        Ok(WorthQueryApplicationAttemptAffinity {
            basis: self.core,
            provider_session,
            product: self.product.publication_binding(),
        })
    }

    pub(super) fn affinity_mismatches(
        &self,
        provider_session: &WorthQueryProviderSessionTerminalBinding,
    ) -> std::collections::BTreeSet<WorthQueryApplicationAttemptAffinityMismatch> {
        let mut mismatches = self.affinity_mismatches_view(provider_session);
        if provider_session
            .application_product_identity()
            .is_none_or(|product| {
                product
                    != &crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                        self.product.observation(),
                    )
            })
        {
            mismatches.insert(WorthQueryApplicationAttemptAffinityMismatch::Product);
        }
        mismatches
    }

    pub fn affinity_mismatches_view(
        &self,
        plan: &impl WorthQueryApplicationAttemptAffinityView,
    ) -> std::collections::BTreeSet<WorthQueryApplicationAttemptAffinityMismatch> {
        use WorthQueryApplicationAttemptAffinityMismatch as M;
        let checks = [
            (plan.runtime_authority() != self.core.runtime, M::Runtime),
            (
                plan.installed_operation() != self.core.installed_operation.as_ref(),
                M::InstalledOperation,
            ),
            (
                plan.resource_binding() != self.core.resource_binding.as_ref(),
                M::ResourceBinding,
            ),
            (
                plan.operation_attempt() != Some(self.core.operation_attempt),
                M::OperationAttempt,
            ),
            (
                plan.operation_slot() != Some(self.core.operation.as_ref()),
                M::OperationSlot,
            ),
            (
                plan.schema_binding() != Some(&self.core.installed_binding),
                M::SchemaBinding,
            ),
            (plan.snapshot() != Some(&self.core.snapshot), M::Snapshot),
            (
                plan.graph_work_session() != Some(self.core.graph_work_session.as_u64()),
                M::GraphWorkSession,
            ),
            (
                plan.graph_work_managed_run() != Some(self.core.graph_work_managed_run.as_u64()),
                M::GraphWorkManagedRun,
            ),
        ];
        checks
            .into_iter()
            .filter_map(|(different, mismatch)| different.then_some(mismatch))
            .collect()
    }
}

impl WorthQueryApplicationAttemptAffinity {
    pub fn take_original_completion_publisher(
        &self,
    ) -> super::super::super::provider::OriginalCompletionPublisher {
        self.basis.completion.take()
    }

    pub fn owns_original_read(&self, read: &super::super::CompletedOriginalRead) -> bool {
        read.belongs_to_admission(self.basis.operation_attempt)
    }

    pub fn binds_original_read(&self, read: &super::super::CompletedOriginalRead) -> bool {
        self.owns_original_read(read) && read.belongs_to_graph_work(self.basis.graph_work_session)
    }
    pub fn admitted_request(
        &self,
    ) -> &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness
    {
        &self.basis.admitted_request
    }
    pub fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        &self.basis.request
    }
    pub fn product_publication(&self) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding{
        &self.product
    }

    pub fn lookup_identity(&self) -> WorthQueryProviderSessionAffinityIdentity {
        self.provider_session.affinity_identity()
    }

    pub const fn provider_session(&self) -> &WorthQueryProviderSessionTerminalBinding {
        &self.provider_session
    }

    pub fn runtime_authority_u64(&self) -> u64 {
        self.basis.runtime.as_u64()
    }

    pub const fn installed_binding(
        &self,
    ) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        &self.basis.installed_binding
    }

    pub fn operation(&self) -> &str {
        &self.basis.operation
    }

    pub fn operation_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        &self.basis.operation_scope
    }

    pub const fn graph_work_session(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.basis.graph_work_session
    }

    pub const fn branch(&self) -> &BranchId {
        &self.basis.branch
    }

    pub fn admits_session(&self, session: WorthQueryProviderSessionView<'_>) -> bool {
        self.provider_session.admits_session_view(session)
    }

    pub fn same_session(&self, binding: &WorthQueryProviderSessionTerminalBinding) -> bool {
        self.provider_session.same_session(binding)
    }

    pub fn admits_cleanup(
        &self,
        cleanup: &crate::domain_computation::provider_session::WorthQueryProvisionalOverlayCleanupBinding,
    ) -> bool {
        self.provider_session.admits_cleanup_binding(cleanup)
    }
}

#[cfg(test)]
#[path = "application_attempt_affinity_inline_type_budget.rs"]
pub mod inline_type_budget;

mod court_doors_for_test;
