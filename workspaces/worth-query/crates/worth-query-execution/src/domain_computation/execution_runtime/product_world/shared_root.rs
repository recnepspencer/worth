use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

use super::WorthQueryProductRuntime;

/// Opaque handle to the one World and sealed Bridge assembled for an
/// application runtime. Retained transport values keep identity but cannot
/// extend either owner's lifecycle.
#[derive(Clone)]
pub struct WorthQueryProductSharedRoot {
    product_identity: Arc<super::runtime::WorthQueryProductRootIdentity>,
    pub(super) bridge: std::sync::Weak<std::sync::RwLock<BridgeSealedRuntimeAssembly>>,
}

impl WorthQueryProductSharedRoot {
    pub(crate) fn new(
        product: WorthQueryProductRuntime,
        bridge: Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Self {
        Self {
            product_identity: product.root_identity(),
            bridge: Arc::downgrade(&bridge),
        }
    }

    #[doc(hidden)]
    pub fn is_same_root_as(&self, candidate: &Self) -> bool {
        Arc::ptr_eq(&self.product_identity, &candidate.product_identity)
            && std::sync::Weak::ptr_eq(&self.bridge, &candidate.bridge)
    }

    #[doc(hidden)]
    pub fn accepts_performed_change(
        &self,
        change: &super::WorthQueryPerformedRelationalProductChange,
    ) -> bool {
        Arc::ptr_eq(&self.product_identity, &change.root_identity)
    }
}
