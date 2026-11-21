//! Descriptive original-principal comparison, never cached authorization.

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::EntityId;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use super::*;
use worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort;

/// Opaque full native principal association retained by Publication commands.
pub struct WorthQueryCommandPrincipalPin {
    entity: EntityId,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
}
impl std::fmt::Debug for WorthQueryCommandPrincipalPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryCommandPrincipalPin")
            .finish_non_exhaustive()
    }
}
impl<S, P, I> WorthQueryAuthenticatedPrincipal<S, P, I> {
    pub fn command_principal_pin(
        &self,
        _: &dyn WorthQueryProgramPublicationAccessPort,
    ) -> WorthQueryCommandPrincipalPin {
        WorthQueryCommandPrincipalPin {
            entity: self.principal_entity_id(),
            runtime: self.runtime_authority(),
            binding: self.binding_identity.clone(),
        }
    }
    pub fn matches_command_principal(
        &self,
        _: &dyn WorthQueryProgramPublicationAccessPort,
        original: &WorthQueryCommandPrincipalPin,
    ) -> bool {
        self.principal_entity_id() == original.entity
            && self.runtime_authority() == original.runtime
            && self.binding_identity == original.binding
    }
}
