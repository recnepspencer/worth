//! `managed_bridge`'s vocabulary, moved whole from `primary_graph::managed_bridge`: its root file's items here, one child per other source file.

use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;
use worth_runtime_bridge::facade::RuntimeBridge;

pub struct WorthQueryInstalledApplicationBridge {
    pub(in crate::domain_computation::primary_graph) ordinary: RuntimeBridge,
    pub(in crate::domain_computation::primary_graph) conditional:
        std::sync::Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>>,
}

impl WorthQueryInstalledApplicationBridge {
    pub(in crate::domain_computation::primary_graph) fn ordinary(&self) -> &RuntimeBridge {
        &self.ordinary
    }

    pub fn conditional(&self) -> std::sync::RwLockReadGuard<'_, BridgeSealedRuntimeAssembly> {
        self.conditional
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(in crate::domain_computation::primary_graph) fn conditional_lifecycle(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, BridgeSealedRuntimeAssembly> {
        self.conditional
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(in crate::domain_computation::primary_graph) fn conditional_operations(
        &self,
    ) -> std::sync::Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>> {
        std::sync::Arc::clone(&self.conditional)
    }
}

impl WorthQueryInstalledApplicationBridge {
    #[doc(hidden)]
    pub fn ordinary_for_test(&self) -> &RuntimeBridge {
        Self::ordinary(self)
    }
}
