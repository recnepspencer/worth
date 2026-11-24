//! Source-contract staging and exact target installation for native recovery.
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
#[path = "field_comparison_request.rs"]
mod physical_field_basis;
pub(crate) use crate::schema::codec_field_comparison::CodecFieldLocus;
pub(crate) use physical_field_basis::codec_field_changes;

use crate::runtime::RelationalRuntime;
use crate::schema::data::RelationalSchemaRegistry;

pub use crate::schema::data::program_activation::RecoveredCheckpointProgramActivationUpdate;

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
            source_groups: self
                .schema_contract_runtime
                .custom_invariant_registries
                .source_inventory(),
            custom_invariant_generation: self.schema_contract_runtime.custom_invariant_generation,
            custom_invariant_inventory_digest: super::custom_invariant_inventory_digest(
                &registrations,
            ),
            retained_entity_kind_count: (*registry.owner_entity_kinds()).len(),
            retained_relation_kind_count: (*registry.owner_relation_kinds()).len(),
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
            &registry,
            &installed.config.schema.invariant_catalog.registrations,
            previous.custom_invariant_registries.clone(),
            previous.custom_invariant_generation,
            previous.initial_custom_invariants_sealed,
            crate::schema::SchemaCompilationControl::Configuration,
        )
        .expect("request-free codec configuration has no execution refusal");
        std::sync::Arc::make_mut(&mut installed.config)
            .schema
            .registry = registry;
        installed.schema_contract_runtime = std::sync::Arc::new(rebuilt);
        drop(installed);
        self.reconfigure(|_| {});
    }
}
