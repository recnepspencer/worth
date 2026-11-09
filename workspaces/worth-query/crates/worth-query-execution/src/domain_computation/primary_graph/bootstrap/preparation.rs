use super::*;

impl WorthQueryExecutionInstallationAuthority {
    pub fn prepare_primary_graph<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        // Installation has no World identity yet; this borrows its admitted root.
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            phase,
            runtime,
            installed_schema,
            RelationalRuntimeApi::builder().build()?,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
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
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
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
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
            None,
            false,
        )
    }

    /// The phase witnesses admission before inline Relational schema and recovery reads.
    pub(super) fn prepare_primary_graph_with_optional_checkpoint<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        checkpoint: Option<&super::super::application_checkpoint::DecodedApplicationCheckpoint>,
        codec_transition: bool,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        self.prepare_native_graph(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            checkpoint,
            codec_transition,
        )
        .and_then(|prepared| {
            prepared.into_bootstrap(runtime, installed_schema, product_world_resources)
        })
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_native_graph<
        Schema: ApplicationSchema,
    >(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        mut relational_runtime: RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        checkpoint: Option<&super::super::application_checkpoint::DecodedApplicationCheckpoint>,
        codec_transition: bool,
    ) -> Result<
        super::prepared_native::PreparedPrimaryGraphInstallation,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        let execution = phase.execution_request();

        let mut schema = self.prepare_native_schema(
            phase,
            runtime,
            installed_schema,
            &mut relational_runtime,
            invariant_factories,
        )?;
        let (recovered_relational_authority, recovered_checkpoint_restore_work) = match checkpoint {
            Some(checkpoint) => {
                let origin = checkpoint.origin.as_ref().ok_or_else(|| primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InterimCheckpointCaptureOriginMissing,
                    "interim checkpoint capture origin is required".to_owned()))?;
                let mut recovery = relational_runtime.durability_recovery();
                let (outcome, authority) = if codec_transition {
                    recovery.restore_interim_checkpoint_for_codec_transition_with_authority(
                        &execution,
                        &origin.native,
                    )
                } else {
                    recovery.restore_interim_checkpoint_with_authority(&execution, &origin.native)
                }
                .map_err(map_checkpoint_recovery_stop)?;
                if !authority.codec_transition_pending() {
                    schema.receipt = relational_runtime
                    .readmit_recovered_initial_schema_installation(schema.receipt)
                    .map_err(|error| {
                        primary_graph_denial(
                            if error.kind() == worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch {
                                WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedRecoveredNativeAuthorityLayout
                            } else { WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected },
                            format!("recovered schema authority denied: {error}"),
                        )
                    })?;
                }
                (Some(authority), outcome.checkpoint_restore_work)
            }
            None => (None, None),
        };
        let source_publisher = if let Some(authority) = recovered_relational_authority
            .as_ref()
            .filter(|authority| authority.codec_transition_pending())
        {
            let (owner, native) = relational_runtime
                .durability_recovery()
                .issue_recovered_codec_source_publication_owner(&execution, authority)
                .map_err(super::checkpoint_transition::codec::map_admission)?;
            Some(
                super::super::root::application_publication::ApplicationPublication::recovered(
                    owner, native,
                ),
            )
        } else {
            None
        };
        let super::native_meaning::InstalledNativeSchemaMeaning {
            managed_views,
            source_initializer,
            layout,
            receipt: invariant_installation_receipt,
            expected_invariant_inventory_digest,
        } = schema.bind_sources(&relational_runtime.main_branch_identity());
        let mut graph = super::prepared_native::PreparedNativeGraph::new(
            relational_runtime,
            layout,
            installed_schema.binding_identity(),
            managed_views,
            checkpoint.is_some(),
        )?;
        if let Some(publisher) = source_publisher {
            graph.application_publication = Some(std::sync::Arc::new(publisher));
        } else {
            graph.bind_application_publication(&invariant_installation_receipt);
        }
        let recovered_publication = checkpoint
            .map(|checkpoint| checkpoint.recover_publication(&execution, &graph))
            .transpose()?;
        Ok(super::prepared_native::PreparedPrimaryGraphInstallation {
            graph,
            meaning: super::prepared_native::PreparedPrimaryGraphMeaning {
                resource_support:
                    super::super::provider::WorthQueryPrimaryGraphResourceSupport::install(
                        runtime
                            .application_query_resource_profile()
                            .maximum_concurrent_graph_work(),
                    ),
                recovered_publication,
                recovered_relational_authority,
                recovered_checkpoint_restore_work,
                invariant_installation_receipt,
                expected_invariant_inventory_digest,
                typed_rows: Default::default(),
                source_initializer,
            },
        })
    }
}

mod checkpoint_recovery_denial;
pub(in crate::domain_computation::primary_graph) use checkpoint_recovery_denial::map_checkpoint_recovery_stop;

mod court_doors_for_test;
