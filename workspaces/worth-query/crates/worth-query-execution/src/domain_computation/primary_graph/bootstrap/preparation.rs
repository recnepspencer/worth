use super::*;

impl WorthQueryExecutionInstallationAuthority {
    pub fn prepare_primary_graph<Schema>(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            runtime,
            installed_schema,
            RelationalRuntimeApi::builder().build(),
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime<Schema>(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        self.prepare_primary_graph_with_optional_checkpoint(
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
            None,
        )
    }

    pub(super) fn prepare_primary_graph_with_optional_checkpoint<Schema>(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        mut relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        checkpoint: Option<&super::super::application_checkpoint::DecodedApplicationCheckpoint>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        if !self.belongs_to(runtime) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime,
                "execution installation authority belongs to another runtime",
            ));
        }
        if runtime.primary_graph().is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::AlreadyInstalled,
                "execution runtime already owns a primary graph",
            ));
        }
        runtime
            .installed_packages()
            .validate_application_schema(installed_schema)
            .map_err(|denial| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                    denial.subject(),
                )
            })?;
        let (layout, additions) = WorthQueryPrimaryGraphLayout::lower(
            installed_schema.installed_declaration(),
            installed_schema.native_contracts(),
            &relational_runtime.config().schema.registry,
        )?;
        let registrations =
            invariant_factories.lower(installed_schema.binding_identity(), &layout)?;
        let expected_invariant_inventory_digest =
            worth_relational::facade::runtime::custom_invariant_inventory_digest(&registrations);
        let mut invariant_installation_receipt = relational_runtime
            .prepare_initial_schema_installation()
            .map_err(map_initial_schema_installation_denial)?
            .install_with_custom_invariants(additions, registrations)
            .map_err(map_initial_schema_installation_denial)?;
        if invariant_installation_receipt.custom_invariant_inventory_digest()
            != &expected_invariant_inventory_digest
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
                "Relational installed an invariant inventory outside the application catalog",
            ));
        }
        let (recovered_relational_authority, recovered_checkpoint_restore_work) =
            if let Some(checkpoint) = checkpoint {
                let (outcome, authority) = relational_runtime
                    .durability_recovery()
                    .restore_native_checkpoint_with_authority(&checkpoint.native)
                    .map_err(|error| {
                        primary_graph_denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                        format!("native checkpoint recovery denied: {error:?}"),
                    )
                    })?;
                invariant_installation_receipt = relational_runtime
                    .readmit_recovered_initial_schema_installation(invariant_installation_receipt)
                    .map_err(|error| {
                        primary_graph_denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                        format!("recovered schema authority denied: {error}"),
                    )
                    })?;
                (Some(authority), outcome.checkpoint_restore_work)
            } else {
                (None, None)
            };
        let graph = checkpoint::primary_graph_for_installation(
            runtime.authority_identity(),
            installed_schema.binding_identity(),
            layout,
            relational_runtime,
            checkpoint.is_some(),
        )?;
        let recovered_publication = checkpoint
            .map(|checkpoint| checkpoint.recover_publication(&graph))
            .transpose()?;
        Ok(WorthQueryPrimaryGraphBootstrap {
            runtime_authority: runtime.authority_identity(),
            installed_packages: runtime.retain_installed_packages(),
            graph,
            product_world_resources,
            rows: Vec::new(),
            external_identities: BTreeSet::new(),
            principal_identities: BTreeSet::new(),
            principal_keys: BTreeSet::new(),
            entity_keys: BTreeSet::new(),
            pending_entity_keys: BTreeSet::new(),
            relation_keys: BTreeSet::new(),
            entity_rows: Vec::new(),
            relation_rows: Vec::new(),
            committed_principal_count: 0,
            committed_entity_count: 0,
            committed_relation_count: 0,
            last_seed_commit_id: None,
            seed_batch_failed: false,
            recovered_publication,
            recovered_relational_authority,
            recovered_checkpoint_restore_work,
            mutation_handlers: Default::default(),
            program_activation_seed: None,
            invariant_installation_receipt,
            expected_invariant_inventory_digest,
            _schema: PhantomData,
        })
    }
}
