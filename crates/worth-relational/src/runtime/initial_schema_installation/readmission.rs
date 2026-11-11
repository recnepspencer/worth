//! One matcher readmits genuine initial A provenance without relabelling B.
use super::*;
use crate::schema::SchemaCompilationControl;

impl RelationalInitialSchemaInstallationReceipt {
    /// Configuration evidence for the same exclusive receiver, before replay.
    /// This does not claim that its current program was the durable birth program.
    pub(crate) fn require_same_home_receiver(
        &self,
        runtime: &RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalInitialSchemaInstallationDenial> {
        let epoch = runtime.configuration_binding().snapshot();
        let contracts = &epoch.schema_contract_runtime;
        let registry = &epoch.config.schema.registry;
        let control = SchemaCompilationControl::Request(*request);
        let schema =
            crate::branch::RelationalBranchRootSchemaAuthority::registry_digest_for_request(
                registry, request,
            )
            .map_err(|cause| {
                RelationalInitialSchemaInstallationDenial::new(
                    RelationalInitialSchemaInstallationDenialKind::Request(cause),
                    "Home receiver schema request stopped",
                )
            })?;
        let inventory = custom_invariant_inventory_digest_for_compilation(
            contracts.custom_invariant_registries.registrations(),
            control,
        )
        .map_err(|cause| {
            RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::Request(cause),
                "Home receiver inventory request stopped",
            )
        })?;
        if self.runtime_instance_id != runtime.runtime_instance_id()
            || !contracts.initial_custom_invariants_sealed
            || contracts.custom_invariant_generation != self.custom_invariant_generation
            || inventory != self.custom_invariant_inventory_digest
            || contracts.custom_invariant_registries.source_inventory() != self.source_groups
            || registry.entity_kinds.len() != self.retained_entity_kind_count
            || registry.relation_kinds.len() != self.retained_relation_kind_count
            || schema != self.retained_schema_authority_digest
        {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch,
                "Home recovery requires this receiver's actual sealed installation",
            ));
        }
        Ok(())
    }

    pub(crate) fn readmit_against(
        self,
        runtime: u64,
        registry: &RelationalSchemaRegistry,
        contracts: &SchemaContractRuntimeSubsystem,
        schema: [u8; 32],
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, RelationalInitialSchemaInstallationDenial> {
        let actual = custom_invariant_inventory_digest_for_compilation(
            contracts.custom_invariant_registries.registrations(),
            control,
        )
        .map_err(|cause| {
            RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::Request(cause),
                "initial authority digest request refused",
            )
        })?;
        if runtime == self.runtime_instance_id
            || contracts.custom_invariant_generation != self.custom_invariant_generation
            || actual != self.custom_invariant_inventory_digest
            || contracts.custom_invariant_registries.source_inventory() != self.source_groups
            || registry.entity_kinds.len() != self.retained_entity_kind_count
            || registry.relation_kinds.len() != self.retained_relation_kind_count
            || schema != self.retained_schema_authority_digest
        {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch,
                "recovered initial schema or invariant authority differs from its original installation",
            ));
        }
        Ok(Self {
            runtime_instance_id: runtime,
            ..self
        })
    }
}

impl crate::durability::authority::RecoveredRelationalRuntimeAuthority {
    /// Pending transitions preserve authentic initial A evidence separately
    /// from the current replay schema. No A receipt is minted from B metadata.
    pub fn readmit_home_initial_schema_installation(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        expected: RelationalInitialSchemaInstallationReceipt,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        use RelationalInitialSchemaInstallationDenial as Denial;
        use RelationalInitialSchemaInstallationDenialKind as Kind;
        request.consult().map_err(|cause| {
            Denial::new(
                Kind::Request(cause.into()),
                "initial readmission request stopped",
            )
        })?;
        let original = self.home_predecessor_contracts(runtime).map_err(|cause| {
            Denial::new(
                Kind::RecoveredAuthorityMismatch,
                format!("foreign Home recovery: {cause:?}"),
            )
        })?;
        let (registry, contracts, schema) = match original {
            Some(original) => (
                original.meaning.schema.registry(),
                original.meaning.contracts.as_ref(),
                original.meaning.schema.authority_digest(),
            ),
            None => {
                let main = runtime.main_branch_identity();
                let root = runtime
                    .history
                    .branch_cell(main.branch_id())
                    .and_then(|cell| cell.root())
                    .ok_or_else(|| {
                        Denial::new(
                            Kind::RecoveredAuthorityMismatch,
                            "recovered initial root unavailable",
                        )
                    })?;
                return expected.readmit_against(
                    runtime.runtime_instance_id(),
                    &runtime.config.schema.registry,
                    &runtime.schema_contract_runtime,
                    root.schema_authority().authority_digest(),
                    SchemaCompilationControl::Request(*request),
                );
            }
        };
        expected.readmit_against(
            runtime.runtime_instance_id(),
            registry,
            contracts,
            schema,
            SchemaCompilationControl::Request(*request),
        )
    }
}
