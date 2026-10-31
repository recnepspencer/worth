use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInspection;
use crate::domain_computation::primary_graph::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

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

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn release_query_conditional_bindings(&mut self) {
        *self
            .conditional_operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.installed_resources.clear();
    }
}

impl<Schema> Drop for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn drop(&mut self) {
        if let Some(mut owner) = self.conditional_cleanup_owner.take() {
            owner.close();
            self.release_query_conditional_bindings();
        }
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
        if self.conditional_cleanup_owner.is_none() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ApplicationHomeOwnsLifecycle,
                "the application Home retains conditional lifecycle custody",
            ));
        }
        if self.installed_resources.cleanup_retained() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::ResourceCleanupRetained,
                "installed resources still own process-local cleanup or recovery custody",
            ));
        }
        let pending = self
            .conditional_operations
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
        self.conditional_cleanup_owner
            .as_mut()
            .expect("cleanup custody was admitted before inspection")
            .close();
        self.release_query_conditional_bindings();
        Ok(before)
    }
}
