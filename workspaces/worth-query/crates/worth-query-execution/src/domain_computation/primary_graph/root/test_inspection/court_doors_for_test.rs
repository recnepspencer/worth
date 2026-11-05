//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn registered_entity_aspect_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
    ) -> Option<worth_relational::facade::schema::DeclaredAspectContractBinding> {
        Self::registered_entity_aspect(self, request_execution, entity, aspect)
    }
}

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn registered_provider_aspect_identities_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> [worth_foundational::facade::AspectIdentity; 3] {
        Self::registered_provider_aspect_identities(self, request_execution)
    }
}
