//! Fixed-width routing data; the typed receiver still checks actual owner affinity.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph::application_query) struct ManagedIndexRegistrationKey
{
    runtime: WorthQueryRuntimeAuthorityIdentity,
    incarnation: ProductBranchIncarnation,
    principal: EntityId,
    scope: EntityId,
    purpose: [u8; 32],
}

#[cfg(test)]
impl ManagedIndexRegistrationKey {
    // Metadata-storage tests only: this key cannot create a typed receiver or READ.
    pub(in crate::domain_computation::primary_graph::application_query) fn storage_key_for_test(
        runtime: WorthQueryRuntimeAuthorityIdentity,
        incarnation: ProductBranchIncarnation,
        scope: EntityId,
        purpose: [u8; 32],
    ) -> Self {
        Self {
            runtime,
            incarnation,
            principal: scope,
            scope,
            purpose,
        }
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(in crate::domain_computation::primary_graph::application_query) fn registration_key(
        &self,
    ) -> ManagedIndexRegistrationKey {
        ManagedIndexRegistrationKey {
            runtime: self.runtime,
            incarnation: self.incarnation,
            principal: self.principal,
            scope: self.scope,
            // The managed purpose includes the nominal Parent binding. The
            // ordinary Query obligation alone cannot distinguish those bindings.
            purpose: *self
                .association
                .access_contract()
                .graph_obligations()
                .identity()
                .bytes(),
        }
    }
}
