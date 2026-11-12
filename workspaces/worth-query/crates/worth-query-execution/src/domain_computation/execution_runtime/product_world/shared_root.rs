use std::sync::Arc;

use crate::domain_computation::primary_graph::managed_bridge::conditional_owner::{
    ConditionalOwner, ConditionalOwnerObservation,
};

use super::WorthQueryProductRuntime;

/// Opaque handle to the one World and sealed Bridge assembled for an
/// application runtime. Retained transport values keep identity but cannot
/// extend either owner's lifecycle.
#[derive(Clone)]
pub struct WorthQueryProductSharedRoot {
    pub(super) owner_identity: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    product_identity: Arc<super::runtime::WorthQueryProductRootIdentity>,
    pub(super) bridge: ConditionalOwnerObservation,
}

impl WorthQueryProductSharedRoot {
    pub(in crate::domain_computation) fn new(
        product: WorthQueryProductRuntime,
        bridge: ConditionalOwner,
    ) -> Self {
        Self {
            owner_identity: product.owner.owner_identity(),
            product_identity: product.root_identity(),
            bridge: bridge.downgrade(),
        }
    }

    #[doc(hidden)]
    pub fn is_same_root_as(&self, candidate: &Self) -> bool {
        Arc::ptr_eq(&self.product_identity, &candidate.product_identity)
            && self.bridge.shares_original(&candidate.bridge)
    }

    #[doc(hidden)]
    pub fn accepts_performed_change(
        &self,
        change: &super::WorthQueryPerformedRelationalProductChange,
    ) -> bool {
        Arc::ptr_eq(&self.product_identity, &change.root_identity)
    }
}
