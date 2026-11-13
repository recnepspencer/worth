use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInspection;
use crate::domain_computation::primary_graph::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

/// The original cleanup owner drops first, with all of its dependent registries
/// and the rest of the Query runtime still alive. Directory leaves owner None.
pub(in crate::domain_computation::primary_graph) struct ConditionalRuntimeCleanup<Schema> {
    owner: Option<OwnedConditionalCleanup>,
    pub(in crate::domain_computation::primary_graph) definitions: crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
    pub(in crate::domain_computation::primary_graph) operations: std::sync::Mutex<
        super::super::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>,
    >,
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
        self.resources.preflight_home_retirement()
    }

    pub(super) fn new(
        owner: Option<OwnedConditionalCleanup>,
        operations: super::super::conditional_operation::WorthQueryConditionalOperationRegistry<
            Schema,
        >,
    ) -> Self {
        Self {
            owner,
            definitions: Default::default(),
            operations: std::sync::Mutex::new(operations),
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
            .operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.conditional_cleanup.resources.clear();
        self.conditional_cleanup.definitions.clear();
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
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ApplicationHomeOwnsLifecycle,
                "the application Home retains conditional lifecycle custody",
            ));
        }
        if self.conditional_cleanup.resources.cleanup_retained() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ResourceCleanupRetained,
                "installed resources still own process-local cleanup or recovery custody",
            ));
        }
        let pending = self
            .conditional_cleanup
            .operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot()
            .pending_managed_cleanup();
        if let Some(kind) = pending {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ManagedCleanupRetained(kind),
                "conditional runtime still owns failed native cleanup",
            ));
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
        let execution = self.runtime.execution.take();
        self.post_conditional_teardown.take_from(&mut self.runtime);
        self.output_teardown.take_from(&mut self.runtime);
        self.workflow_teardown.take_from(&mut self.runtime);
        if let Some(owner) = &mut self.conditional_cleanup.owner {
            owner.close();
        }
        self.release_query_conditional_bindings();
        drop(execution);
    }
}
