//! Prepare receipt DATA privately; only the acknowledged receiver swap issues it.
use super::*;

impl RelationalInitialSchemaInstallationReceipt {
    pub(in crate::runtime) fn admits_home_target(
        &self,
        runtime: u64,
        schema: &crate::branch::RelationalBranchRootSchemaAuthority,
        contracts: &SchemaContractRuntimeSubsystem,
    ) -> bool {
        self.runtime_instance_id == runtime
            && self.custom_invariant_generation == contracts.custom_invariant_generation
            && self.retained_schema_authority_digest == schema.authority_digest()
            && self.retained_entity_kind_count == schema.registry().entity_kinds.len()
            && self.retained_relation_kind_count == schema.registry().relation_kinds.len()
            && Arc::ptr_eq(
                &self.source_groups,
                &contracts.custom_invariant_registries.source_inventory(),
            )
    }

    pub(in crate::runtime) fn prepare_home_target(
        runtime: u64,
        schema: &crate::branch::RelationalBranchRootSchemaAuthority,
        contracts: &SchemaContractRuntimeSubsystem,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::runtime::RelationalHomeProgramContractDenial> {
        let inventory = custom_invariant_inventory_digest_for_compilation(
            contracts.custom_invariant_registries.registrations(),
            crate::schema::SchemaCompilationControl::Request(*request),
        )
        .map_err(crate::runtime::RelationalHomeProgramContractDenial::Execution)?;
        Ok(Self {
            runtime_instance_id: runtime,
            source_groups: contracts.custom_invariant_registries.source_inventory(),
            custom_invariant_generation: contracts.custom_invariant_generation,
            custom_invariant_inventory_digest: inventory,
            retained_entity_kind_count: schema.registry().entity_kinds.len(),
            retained_relation_kind_count: schema.registry().relation_kinds.len(),
            retained_schema_authority_digest: schema.authority_digest(),
        })
    }
}
