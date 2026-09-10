use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

#[derive(Debug)]
struct GranularInvalidationRuntimeSeal {
    generation: AtomicU64,
}

/// Primary-graph installation identity for granular invalidation collection.
///
/// This identity locates the composition owner. It is not Bridge, Signal, or
/// Query admission authority.
#[derive(Clone)]
pub struct WorthQueryGranularInvalidationInstallation {
    binding_identity: ApplicationSchemaBindingIdentity,
    integration: super::super::WorthQueryPrimaryGraphIntegrationHandle,
    runtime_seal: Arc<GranularInvalidationRuntimeSeal>,
    generation: u64,
    product_root:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot,
}

impl WorthQueryGranularInvalidationInstallation {
    pub(in crate::domain_computation::primary_graph) fn new(
        binding_identity: ApplicationSchemaBindingIdentity,
        integration: super::super::WorthQueryPrimaryGraphIntegrationHandle,
        product_root: crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot,
    ) -> Self {
        Self {
            binding_identity,
            integration,
            runtime_seal: Arc::new(GranularInvalidationRuntimeSeal {
                generation: AtomicU64::new(1),
            }),
            generation: 1,
            product_root,
        }
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    /// Retains Query's opaque access to the same primary graph. This handle
    /// carries graph ownership only; it cannot admit invalidation work.
    #[doc(hidden)]
    pub fn retain_primary_graph_integration_handle(
        &self,
    ) -> super::super::WorthQueryPrimaryGraphIntegrationHandle {
        self.integration.clone()
    }

    /// Retains Query's opaque operation handle to the application's exact
    /// World and sealed Bridge composition.
    #[doc(hidden)]
    pub fn retain_product_shared_root(
        &self,
    ) -> crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot
    {
        self.product_root.clone()
    }

    /// Returns whether this installation minted the execution-owned batch.
    ///
    /// Equality of schema names or canonical digests is intentionally
    /// insufficient: restored and replacement runtimes mint a new seal.
    pub fn admits_batch(&self, batch: &super::WorthQueryGranularInvalidationDeliveryBatch) -> bool {
        self.binding_identity == batch.installation().binding_identity
            && Arc::ptr_eq(&self.runtime_seal, &batch.installation().runtime_seal)
            && self.generation == batch.installation().generation
            && self.generation == self.runtime_seal.generation.load(Ordering::Acquire)
    }

    /// Returns whether both handles name the same current primary runtime.
    ///
    /// This is an observational identity check for composition owners. It
    /// grants no invalidation, readiness, execution, or publication authority.
    #[doc(hidden)]
    pub fn is_same_current_runtime_as(&self, candidate: &Self) -> bool {
        self.binding_identity == candidate.binding_identity
            && Arc::ptr_eq(&self.runtime_seal, &candidate.runtime_seal)
            && self.generation == candidate.generation
            && self.generation == self.runtime_seal.generation.load(Ordering::Acquire)
    }

    /// Returns whether this handle is the exact next generation of the same
    /// runtime lineage. This is an observational rebind check, not delivery
    /// or execution authority.
    #[doc(hidden)]
    pub fn is_immediate_successor_of(&self, previous: &Self) -> bool {
        self.binding_identity == previous.binding_identity
            && Arc::ptr_eq(&self.runtime_seal, &previous.runtime_seal)
            && self.generation == previous.generation.saturating_add(1)
            && self.generation == self.runtime_seal.generation.load(Ordering::Acquire)
    }

    pub(in crate::domain_computation::primary_graph) fn current(&self) -> Self {
        Self {
            binding_identity: self.binding_identity.clone(),
            integration: self.integration.clone(),
            runtime_seal: Arc::clone(&self.runtime_seal),
            generation: self.runtime_seal.generation.load(Ordering::Acquire),
            product_root: self.product_root.clone(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn advance_runtime_generation(&self) {
        self.runtime_seal.generation.fetch_add(1, Ordering::AcqRel);
    }
}
