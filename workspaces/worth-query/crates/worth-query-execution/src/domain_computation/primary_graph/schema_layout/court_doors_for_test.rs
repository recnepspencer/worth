//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn provider_dispatch_outbox_for_test(&self) -> &WorthQueryDispatchOutboxLayout {
        Self::provider_dispatch_outbox(self)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn lower_for_test( schema: &ErasedApplicationSchemaDeclaration, native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog, existing_registry: &RelationalSchemaRegistry) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        Self::lower(schema, native_contracts, existing_registry)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn field_locator_for_test(
        &self, entity: &str, aspect: &str, field: &str) -> Option<&AspectFieldLocator> {
        Self::field_locator(self, entity, aspect, field)
    }
}

impl WorthQueryPrimaryRelationLayout {
    #[doc(hidden)]
    pub fn kind_for_test(&self) -> &KindId {
        &self.kind
    }
}
