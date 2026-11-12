//! Retain the existing installed handle; no erased authority is reconstructed.
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;

mod sealed {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait InstalledOperationSource: sealed::Sealed + Send + Sync {
    fn installed_view(&self) -> InstalledOperationView<'_>;
}

impl sealed::Sealed for super::backing::InstalledOperationBacking {}
impl InstalledOperationSource for super::backing::InstalledOperationBacking {
    fn installed_view(&self) -> InstalledOperationView<'_> {
        self.view()
    }
}

/// Installation bookkeeping retaining actual installed handles, shared before registration.
#[doc(hidden)]
#[derive(Default)]
pub struct InstalledOperationCatalog {
    operations: Mutex<BTreeMap<String, Arc<dyn InstalledOperationSource>>>,
}

impl std::fmt::Debug for InstalledOperationCatalog {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstalledOperationCatalog")
            .finish_non_exhaustive()
    }
}

impl InstalledOperationCatalog {
    /// Allocating cold-readmission snapshot of the actual installed handles.
    /// Fresh verifier configuration is not the authority for persisted limits.
    pub fn readmission_snapshot(&self) -> Vec<Arc<dyn InstalledOperationSource>> {
        self.operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(Arc::clone)
            .collect()
    }

    /// O(log installed operations); a shallow retain, without allocation or compilation.
    pub fn operation(&self, name: &str) -> Option<Arc<dyn InstalledOperationSource>> {
        self.operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
            .map(Arc::clone)
    }

    pub(crate) fn register(
        &self,
        operation: Arc<dyn InstalledOperationSource>,
    ) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
        let mut operations = self
            .operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let view = operation.installed_view();
        if let Some(existing) = operations.get(view.operation()) {
            if !view.same_installation(existing.installed_view()) {
                return Err(super::super::installed_contract_support::operation_denial(
                    super::super::WorthQueryApplicationOperationInstallationDenialKind::OperationMeaningChanged,
                    view.operation(),
                ));
            }
        } else {
            operations.insert(view.operation().to_owned(), operation);
        }
        Ok(())
    }
}

impl super::backing::InstalledOperationBacking {
    fn retain(&self) -> Arc<dyn InstalledOperationSource> {
        Arc::new(self.clone())
    }
}

impl<S, O, I> WorthQueryInstalledApplicationOperation<S, O, I> {
    pub(super) fn retain_for_reentry(&self) -> Arc<dyn InstalledOperationSource> {
        self.core.retain()
    }
}
