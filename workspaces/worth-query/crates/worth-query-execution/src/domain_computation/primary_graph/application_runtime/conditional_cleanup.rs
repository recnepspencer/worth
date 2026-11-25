use std::sync::{Arc, Mutex, RwLock};
use worth_query_execution_query::facade::ports::ConditionalRuntimeInstallationDenialInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInspection;
use crate::domain_computation::primary_graph::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

pub(in crate::domain_computation::primary_graph) struct ConditionalRoutes<Schema> {
    pub(in crate::domain_computation::primary_graph) definitions: RwLock<
        crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
    >,
    pub(in crate::domain_computation::primary_graph) operations: Mutex<
        super::super::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>,
    >,
    pub(in crate::domain_computation::primary_graph) refuse_next_decision_publication:
        std::sync::atomic::AtomicBool,
}

impl<Schema> ConditionalRoutes<Schema> {
    fn new(
        operations: super::super::conditional_operation::WorthQueryConditionalOperationRegistry<
            Schema,
        >,
    ) -> Self {
        Self {
            definitions: RwLock::new(Default::default()),
            operations: Mutex::new(operations),
            refuse_next_decision_publication: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    worth_query_execution_query::facade::ports::ConditionalDefinitionAdmissionPort
    for ConditionalRoutes<Schema>
{
    fn resolve_predecessor(
        &self,
        bridge_root: &worth_query_execution_contracts::facade::ConditionalOwner,
        product: &crate::basis::WorthQueryProductBranchLease,
        binding_identity: &str,
        lease: &Arc<
            crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::ConditionalClockLease,
        >,
    ) -> Result<
        Result<
            (
                Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
                bool,
            ),
            worth_runtime_bridge::facade::BridgeConditionalDenial,
        >,
        crate::domain_computation::primary_graph::WorthQueryApplicationConditionalDefinitionAdmissionDenial,
    >{
        use crate::domain_computation::primary_graph::WorthQueryApplicationConditionalDefinitionAdmissionDenial as Denial;
        let bridge = bridge_root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let definitions = self
            .definitions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(route) = definitions.route(binding_identity, lease) {
            return Ok(route
                .admit_signal_basis(&bridge, product.signal_basis())
                .map(|binding| (binding.installed_lowering(), true)));
        }
        drop(definitions);
        let operation = self
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .admit_clock(binding_identity, lease)
            .ok_or(Denial::ForeignConditionalOperation)?;
        let resolved = match bridge.with_view(product.signal_basis(), |view| {
            operation.admit_operation_binding(&view, product.signal_basis())
        }) {
            Ok(Ok((binding, authored))) => Ok((binding.installed_lowering(), authored)),
            Ok(Err(cause)) | Err(cause) => Err(cause),
        };
        Ok(resolved)
    }
}

/// The original cleanup owner drops first, with all of its dependent registries
/// and the rest of the Query runtime still alive. Directory leaves owner None.
pub(in crate::domain_computation::primary_graph) struct ConditionalRuntimeCleanup<Schema> {
    owner: Option<OwnedConditionalCleanup>,
    pub(in crate::domain_computation::primary_graph) routes: Arc<ConditionalRoutes<Schema>>,
    pub(in crate::domain_computation::primary_graph) resources:
        super::super::application_resource::installation::InstalledResourceRegistry<Schema>,
}

impl<Schema> ConditionalRuntimeCleanup<Schema> {
    pub(super) fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::WorthQueryHomeRetirementDenial> {
        use super::{
            WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
        };
        request.consult().map_err(Denial::Execution)?;
        if self.owner.is_some() {
            return Err(Denial::Unsupported(Owner::ConditionalRuntime));
        }
        Ok(())
    }

    pub(super) fn new(
        owner: Option<OwnedConditionalCleanup>,
        operations: super::super::conditional_operation::WorthQueryConditionalOperationRegistry<
            Schema,
        >,
    ) -> Self {
        Self {
            owner,
            routes: Arc::new(ConditionalRoutes::new(operations)),
            resources: Default::default(),
        }
    }
}

/// Actual cleanup custody for an assembly constructed by Query. Adopting a
/// Home's operational Bridge references leaves this custody with Store.
pub(super) struct OwnedConditionalCleanup {
    assembly: std::sync::Arc<
        std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
    >,
}

impl OwnedConditionalCleanup {
    pub(super) fn new(
        assembly: std::sync::Arc<
            std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
        >,
    ) -> Self {
        Self { assembly }
    }

    fn close(&mut self) {
        self.assembly
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .close_conditional_resources();
    }
}

impl Drop for OwnedConditionalCleanup {
    fn drop(&mut self) {
        self.close();
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn release_query_conditional_bindings(&mut self) {
        *self
            .conditional_cleanup
            .routes
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.conditional_cleanup.resources.clear();
        self.conditional_cleanup
            .routes
            .definitions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn close_conditional_runtime(
        &mut self,
    ) -> Result<
        WorthQueryConditionalRuntimeInspection,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        if self.conditional_cleanup.owner.is_none() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::from_kind_and_subject(
                WorthQueryConditionalRuntimeInstallationDenialKind::ApplicationHomeOwnsLifecycle,
                "the application Home retains conditional lifecycle custody",
            ));
        }
        if self.conditional_cleanup.resources.cleanup_retained() {
            return Err(
                WorthQueryConditionalRuntimeInstallationDenial::from_kind_and_subject(
                    WorthQueryConditionalRuntimeInstallationDenialKind::ResourceCleanupRetained,
                    "installed resources still own process-local cleanup or recovery custody",
                ),
            );
        }
        let pending = self
            .conditional_cleanup
            .routes
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot()
            .pending_managed_cleanup();
        if let Some(kind) = pending {
            return Err(
                WorthQueryConditionalRuntimeInstallationDenial::from_kind_and_subject(
                    WorthQueryConditionalRuntimeInstallationDenialKind::ManagedCleanupRetained(
                        kind,
                    ),
                    "conditional runtime still owns failed native cleanup",
                ),
            );
        }
        let before = self.inspect_conditional_runtime();
        self.conditional_cleanup
            .owner
            .as_mut()
            .expect("cleanup custody was admitted before inspection")
            .close();
        self.release_query_conditional_bindings();
        Ok(before)
    }
}

impl<Schema> Drop for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn drop(&mut self) {
        // The local owner drops first even if cleanup unwinds. Move later
        // custody into its original field positions before cleanup can panic.
        use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
        let execution = self.inner.runtime.execution_option_mut().take();
        self.inner
            .post_conditional_teardown
            .take_from(&mut self.inner.runtime);
        self.inner
            .output_teardown
            .take_from(&mut self.inner.runtime);
        self.inner
            .workflow_teardown
            .take_from(&mut self.inner.runtime);
        if let Some(owner) = &mut self.conditional_cleanup.owner {
            owner.close();
        }
        self.release_query_conditional_bindings();
        drop(execution);
    }
}
