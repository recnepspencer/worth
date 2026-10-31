//! Exclusive installation custody ends before the operational source is shared.
use super::*;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;

pub(in crate::domain_computation::primary_graph) struct PreparedNativeGraph {
    pub(super) native: RelationalRuntime,
    pub(in crate::domain_computation::primary_graph) layout: Arc<WorthQueryPrimaryGraphLayout>,
    binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) application_publication:
        Option<Arc<super::super::root::application_publication::ApplicationPublication>>,
}

impl PreparedNativeGraph {
    pub(super) fn new(
        native: RelationalRuntime,
        mut layout: WorthQueryPrimaryGraphLayout,
        binding: ApplicationSchemaBindingIdentity,
        recovered: bool,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        use super::super::root::index_installation::{
            register_primary_graph_indexes, IndexInstallationPosture,
        };
        let posture = if recovered {
            IndexInstallationPosture::RequireRecovered
        } else {
            IndexInstallationPosture::Register
        };
        register_primary_graph_indexes(&mut layout, &native, posture)
            .map_err(super::checkpoint::checkpoint_denial)?;
        Ok(Self {
            native,
            layout: Arc::new(layout),
            binding,
            application_publication: None,
        })
    }

    pub(super) fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub(super) fn with_runtime<T>(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        read(&self.native)
    }

    pub(super) fn bind_application_publication(
        &mut self,
        receipt: &RelationalInitialSchemaInstallationReceipt,
    ) {
        self.application_publication = Some(Arc::new(
            super::super::root::application_publication::ApplicationPublication::installed(receipt),
        ));
    }

    pub(super) fn primary_index_ids(
        &self,
    ) -> Arc<[worth_relational::facade::indexes::DerivedIndexId]> {
        super::super::root::index_installation::primary_index_inventory(&self.layout)
    }
}

pub(in crate::domain_computation::primary_graph) struct PreparedPrimaryGraphInstallation {
    pub(in crate::domain_computation::primary_graph) graph: PreparedNativeGraph,
    pub(in crate::domain_computation::primary_graph) resource_support:
        super::super::provider::WorthQueryPrimaryGraphResourceSupport,
    pub(in crate::domain_computation::primary_graph) recovered_publication:
        Option<WorthQueryPrimaryGraphPublication>,
    pub(in crate::domain_computation::primary_graph) recovered_relational_authority:
        Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    pub(in crate::domain_computation::primary_graph) recovered_checkpoint_restore_work:
        Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    pub(in crate::domain_computation::primary_graph) invariant_installation_receipt:
        RelationalInitialSchemaInstallationReceipt,
    pub(in crate::domain_computation::primary_graph) expected_invariant_inventory_digest: [u8; 32],
    pub(in crate::domain_computation::primary_graph) typed_rows:
        super::super::typed_bootstrap::TypedBootstrapRows,
}

impl PreparedPrimaryGraphInstallation {
    pub(in crate::domain_computation::primary_graph) fn resource_support_ref(&self) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot{
        self.resource_support.snapshot_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn has_pending_codec_transition(
        &self,
    ) -> bool {
        self.recovered_relational_authority
            .as_ref()
            .is_some_and(|authority| authority.codec_transition_pending())
    }

    pub(in crate::domain_computation::primary_graph) fn into_bootstrap<
        Schema: ApplicationSchema,
    >(
        self,
        runtime: &WorthQueryExecutionRuntime,
        installed: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    {
        if self.has_pending_codec_transition() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                "exclusive codec transition must finish before sharing the installed source",
            ));
        }
        let PreparedNativeGraph {
            native,
            layout,
            binding,
            application_publication,
        } = self.graph;
        let mut graph = WorthQueryPrimaryGraph::from_installed_native(
            runtime.authority_identity(),
            binding,
            layout,
            native,
            product_world_resources.invalidation_resources(),
        );
        graph.application_publication = application_publication;
        Ok(WorthQueryPrimaryGraphBootstrap {
            value_bindings: installed.value_bindings().clone(),
            runtime_authority: runtime.authority_identity(),
            installed_packages: runtime.retain_installed_packages(),
            graph,
            resource_support: self.resource_support,
            product_world_resources,
            rows: Vec::new(),
            external_identities: BTreeSet::new(),
            principal_identities: BTreeSet::new(),
            typed_rows: self.typed_rows,
            principal_keys: BTreeSet::new(),
            committed_principal_count: 0,
            committed_entity_count: 0,
            committed_relation_count: 0,
            last_seed_commit_id: None,
            seed_batch_failed: false,
            recovered_publication: self.recovered_publication,
            recovered_relational_authority: self.recovered_relational_authority,
            recovered_checkpoint_restore_work: self.recovered_checkpoint_restore_work,
            mutation_handlers: Default::default(),
            resources: Default::default(),
            managed_indexes: Default::default(),
            program_activation_seed: None,
            program_activation: None,
            invariant_installation_receipt: self.invariant_installation_receipt,
            expected_invariant_inventory_digest: self.expected_invariant_inventory_digest,
            _schema: PhantomData,
        })
    }
}
