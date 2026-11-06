//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn provider_dispatch_outbox_for_test(&self) -> &WorthQueryDispatchOutboxLayout {
        Self::provider_dispatch_outbox(self)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn field_locator_for_test(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectFieldLocator> {
        Self::field_locator(self, entity, aspect, field)
    }
}

impl WorthQueryPrimaryRelationLayout {
    #[doc(hidden)]
    pub fn kind_for_test(&self) -> &KindId {
        &self.kind
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn field_for_test(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        Self::field(self, entity, aspect, field)
    }
}

impl WorthQueryPrimaryFieldLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorthQueryPrimaryFieldLayout {
    #[doc(hidden)]
    pub fn equality_index_id_for_test(&self) -> &Option<DerivedIndexId> {
        &self.equality_index_id
    }
}

impl WorthQueryPrimaryFieldLayout {
    #[doc(hidden)]
    pub fn locator_for_test(&self) -> &AspectFieldLocator {
        &self.locator
    }
}

impl WorthQueryPrimaryFieldLayout {
    #[doc(hidden)]
    pub fn locator_into_for_test(self) -> AspectFieldLocator {
        self.locator
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn aspect_contracts_for_test(
        &self,
    ) -> &BTreeMap<String, BTreeMap<AspectKey, DeclaredAspectContractBinding>> {
        &self.aspect_contracts
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn entity_kinds_for_test(&self) -> &BTreeMap<String, KindId> {
        &self.entity_kinds
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn provider_idempotency_for_test(&self) -> &WorthQueryProviderIdempotencyLayout {
        Self::provider_idempotency(self)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn aspect_contracts_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<String, BTreeMap<AspectKey, DeclaredAspectContractBinding>> {
        &mut self.aspect_contracts
    }
}

impl WorthQueryPrimaryFieldLayout {
    #[doc(hidden)]
    pub fn equality_index_id_mut_for_test(&mut self) -> &mut Option<DerivedIndexId> {
        &mut self.equality_index_id
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn lower_for_test(
        schema: &ErasedApplicationSchemaDeclaration,
        native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        existing_registry: &RelationalSchemaRegistry,
        value_bindings: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Result<(Self, RelationalSchemaRegistry), WorthQueryPrimaryGraphInstallationDenial> {
        Self::lower(schema, native_contracts, existing_registry, value_bindings)
    }
}
