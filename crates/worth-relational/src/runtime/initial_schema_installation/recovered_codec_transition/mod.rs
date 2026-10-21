//! Source-contract staging and exact target installation for native recovery.
mod physical_field_basis;
pub(crate) use physical_field_basis::{
    codec_field_changes, codec_field_changes_in_context, CodecFieldComparisonStop, CodecFieldLocus,
};

use crate::runtime::RelationalRuntime;
use crate::schema::data::RelationalSchemaRegistry;

/// Exact program activation change authenticated by Query's installed owner.
/// This description is joined to that owner's occurrence proof and the
/// recovered native source root before it permits any protected-row change.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveredCheckpointProgramActivationUpdate {
    pub entity_id: crate::identity::data::EntityId,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
    pub field_locator: worth_foundational::facade::AspectFieldLocator,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
    pub expected: worth_foundational::facade::AspectValue,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
    pub replacement: worth_foundational::facade::AspectValue,
}

impl RelationalRuntime {
    pub(crate) fn issue_recovered_codec_source_publication_owner(
        &self,
    ) -> (
        worth_proof::InstalledApplicationPublicationOwner,
        crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    ) {
        let registrations = self
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let registry = &self.config().schema.registry;
        super::RelationalInitialSchemaInstallationReceipt {
            runtime_instance_id: self.runtime_instance_id(),
            custom_invariant_generation: self.schema_contract_runtime.custom_invariant_generation,
            custom_invariant_inventory_digest: super::custom_invariant_inventory_digest(
                &registrations,
            ),
            retained_entity_kind_count: registry.entity_kinds.len(),
            retained_relation_kind_count: registry.relation_kinds.len(),
            retained_schema_authority_digest:
                crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &registry.authority_snapshot(),
                ),
        }
        .issue_application_publication_owner()
    }

    /// Recovery calls this after exact native target settlement. Branch roots
    /// already own matching target contracts and prepared index generations.
    pub(crate) fn install_recovered_codec_target(&mut self, registry: RelationalSchemaRegistry) {
        let configuration = self.configuration_binding();
        let mut installed = configuration.initial_installation();
        let previous = &installed.schema_contract_runtime;
        let rebuilt = super::rebuilt_schema_contract_runtime(
            &installed,
            registry.clone(),
            previous.custom_invariant_registries.clone(),
            previous.custom_invariant_generation,
            previous.initial_custom_invariants_sealed,
        );
        std::sync::Arc::make_mut(&mut installed.config)
            .schema
            .registry = registry;
        installed.schema_contract_runtime = std::sync::Arc::new(rebuilt);
        drop(installed);
        self.reconfigure(|_| {});
    }
}
