//! Exact owner-minted association for one application provider attempt.

use worth_relational::facade::{history::BranchId, snapshots::SnapshotHandle};

use crate::domain_computation::{
    authorization::{WorthQueryAdmittedApplicationOperation, WorthQueryOperationAdmissionIdentity},
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
pub(in crate::domain_computation) struct WorthQueryApplicationAttemptBasis {
    core: WorthQueryApplicationAttemptCore,
    product: crate::basis::WorthQueryProductBranchLease,
}

struct WorthQueryApplicationAttemptCore {
    runtime: WorthQueryRuntimeAuthorityIdentity,
    operation_attempt: WorthQueryOperationAdmissionIdentity,
    installed_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    installed_operation: std::sync::Arc<str>,
    resource_binding: std::sync::Arc<str>,
    operation: std::sync::Arc<str>,
    snapshot: SnapshotHandle,
    graph_work_session: WorthQueryGraphWorkSessionIdentity,
    graph_work_managed_run: WorthQueryGraphWorkManagedRunIdentity,
    branch: BranchId,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

/// Inseparable authority association retained by the provider attempt store.
///
/// No descriptive identity or lower provider-session identity can construct
/// this value. It is minted only after the exact provider plan is checked
/// against the already-admitted operation, pinned snapshot, graph-work session,
/// and Relational branch.
pub(in crate::domain_computation) struct WorthQueryApplicationAttemptAffinity {
    basis: WorthQueryApplicationAttemptCore,
    provider_session: WorthQueryProviderSessionTerminalBinding,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph::application_attempt) enum WorthQueryApplicationAttemptAffinityMismatch
{
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

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) trait WorthQueryApplicationAttemptAffinityView
{
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
    pub(in crate::domain_computation::primary_graph::application_attempt) fn capture<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        application: &super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        snapshot: &SnapshotHandle,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, ()> {
        if admission.runtime_authority() != application.runtime.authority_identity()
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
            return Err(());
        }
        Ok(Self {
            core: WorthQueryApplicationAttemptCore {
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
            },
            product: product.retained_clone(),
        })
    }

    pub(in crate::domain_computation) fn retained_product(
        &self,
    ) -> crate::basis::WorthQueryProductBranchLease {
        self.product.retained_clone()
    }

    pub(in crate::domain_computation) fn bind_live_session(
        self,
        provider_session: &WorthQueryProviderSessionAffinity<'_>,
    ) -> Result<WorthQueryApplicationAttemptAffinity, ()> {
        let provider_session = provider_session.terminal_binding();
        if !self.affinity_mismatches(&provider_session).is_empty()
            || provider_session
                .application_product()
                .is_none_or(|product| product.observation() != self.product.observation())
        {
            return Err(());
        }
        Ok(WorthQueryApplicationAttemptAffinity {
            basis: self.core,
            provider_session,
        })
    }

    pub(super) fn affinity_mismatches(
        &self,
        provider_session: &WorthQueryProviderSessionTerminalBinding,
    ) -> std::collections::BTreeSet<WorthQueryApplicationAttemptAffinityMismatch> {
        let mut mismatches = self.affinity_mismatches_view(provider_session);
        if provider_session
            .application_product()
            .is_none_or(|product| product.observation() != self.product.observation())
        {
            mismatches.insert(WorthQueryApplicationAttemptAffinityMismatch::Product);
        }
        mismatches
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn affinity_mismatches_view(
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
    pub(in crate::domain_computation::primary_graph) fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        &self.basis.request
    }
    pub(in crate::domain_computation::primary_graph) fn product_publication(&self) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding{
        self.provider_session
            .application_product()
            .expect("application attempt affinity retains its selected product")
    }

    pub(in crate::domain_computation::primary_graph) fn lookup_identity(
        &self,
    ) -> WorthQueryProviderSessionAffinityIdentity {
        self.provider_session.affinity_identity()
    }

    pub(in crate::domain_computation::primary_graph) const fn provider_session(
        &self,
    ) -> &WorthQueryProviderSessionTerminalBinding {
        &self.provider_session
    }

    pub(in crate::domain_computation::primary_graph) const fn installed_binding(
        &self,
    ) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        &self.basis.installed_binding
    }

    pub(in crate::domain_computation::primary_graph) fn operation(&self) -> &str {
        &self.basis.operation
    }

    pub(in crate::domain_computation::primary_graph) const fn graph_work_session(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.basis.graph_work_session
    }

    pub(in crate::domain_computation::primary_graph) const fn branch(&self) -> &BranchId {
        &self.basis.branch
    }

    pub(in crate::domain_computation::primary_graph) fn admits_session(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.provider_session.admits_session_view(session)
    }

    pub(in crate::domain_computation::primary_graph) fn same_session(
        &self,
        binding: &WorthQueryProviderSessionTerminalBinding,
    ) -> bool {
        self.provider_session.same_session(binding)
    }

    pub(in crate::domain_computation::primary_graph) fn admits_cleanup(
        &self,
        cleanup: &crate::domain_computation::provider_session::WorthQueryProvisionalOverlayCleanupBinding,
    ) -> bool {
        self.provider_session.admits_cleanup_binding(cleanup)
    }
}
