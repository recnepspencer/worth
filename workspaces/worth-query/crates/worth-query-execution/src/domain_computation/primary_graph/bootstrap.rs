use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationIdentityScalarValueBinding, ApplicationSchema, WorthQueryExternalPrincipalIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPackageIndex,
    WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalMappingStatus,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::storage::{
    authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
};

use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
    WorthQueryRuntimeAuthorityIdentity,
};

use super::bootstrap_publication::{build_identity_indexes, commit_bootstrap_rows};
use super::initial_schema_denial::map_initial_schema_installation_denial;
use super::schema_layout::{WorthQueryPrimaryGraphLayout, WorthQueryPrimaryPrincipalBindingLayout};
use super::WorthQueryApplicationInvariantFactories;
use super::{
    WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraph,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

mod binding_denial;
mod publication;
mod publication_target;
use binding_denial::map_binding_denial_kind;
mod truth_partition;
pub use publication::WorthQueryPrimaryGraphPublication;

pub(super) struct WorthQueryPrincipalBootstrapRow {
    pub(super) binding: String,
    pub(super) principal_key: String,
    pub(super) principal_identity: worth_foundational::facade::AspectValue,
    pub(super) identity: WorthQueryExternalPrincipalIdentity,
    pub(super) status: WorthQueryPrincipalMappingStatus,
    pub(super) layout: WorthQueryPrimaryPrincipalBindingLayout,
}

/// Move-only installation phase for the primary graph.
///
/// Publishing consumes this value. No principal bootstrap mutation surface is
/// retained by the execution runtime.
pub struct WorthQueryPrimaryGraphBootstrap<Schema> {
    pub(super) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    installed_packages: Arc<WorthQueryInstalledPackageIndex>,
    pub(super) graph: WorthQueryPrimaryGraph,
    pub(super) product_world_resources:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    rows: Vec<WorthQueryPrincipalBootstrapRow>,
    external_identities: BTreeSet<(String, WorthQueryExternalPrincipalIdentity)>,
    principal_identities: BTreeSet<(String, AuthoritativeFieldComparisonKey)>,
    principal_keys: BTreeSet<(KindId, String)>,
    pub(super) entity_keys: BTreeSet<(KindId, String)>,
    pub(super) relation_keys: BTreeSet<(KindId, String)>,
    pub(super) entity_rows: Vec<super::typed_bootstrap::WorthQueryTypedEntityBootstrapRow>,
    pub(super) relation_rows: Vec<super::typed_bootstrap::WorthQueryTypedRelationBootstrapRow>,
    pub(super) mutation_handlers: super::handler::PendingMutationHandlerRegistry<Schema>,
    invariant_installation_receipt:
        worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt,
    expected_invariant_inventory_digest: [u8; 32],
    _schema: PhantomData<fn() -> Schema>,
}

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
        mut relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
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
        let invariant_installation_receipt = relational_runtime
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
        let graph = WorthQueryPrimaryGraph::new(
            runtime.authority_identity(),
            installed_schema.binding_identity(),
            layout,
            relational_runtime,
        );
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
            relation_keys: BTreeSet::new(),
            entity_rows: Vec::new(),
            relation_rows: Vec::new(),
            mutation_handlers: Default::default(),
            invariant_installation_receipt,
            expected_invariant_inventory_digest,
            _schema: PhantomData,
        })
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn bind_principal<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &mut self,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        principal_key: WorthQueryApplicationPrincipalKey<Schema, Principal>,
        principal_identity: PrincipalIdentity,
        identity: WorthQueryExternalPrincipalIdentity,
        status: WorthQueryPrincipalMappingStatus,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.installed_packages
            .validate_principal_binding(installed_binding)
            .map_err(|denial| {
                primary_graph_denial(map_binding_denial_kind(denial.kind()), denial.binding())
            })?;
        if installed_binding.binding_identity() != self.graph.binding_identity() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::BindingSchemaMismatch,
                installed_binding.binding(),
            ));
        }
        let layout = self
            .graph
            .layout
            .principal_binding(installed_binding.binding())
            .cloned()
            .ok_or_else(|| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::BindingNotInstalled,
                    installed_binding.binding(),
                )
            })?;
        self.admit_principal_row(WorthQueryPrincipalBootstrapRow {
            binding: installed_binding.binding().to_string(),
            principal_key: principal_key.as_str().to_string(),
            principal_identity: installed_binding
                .principal_identity_binding()
                .encode(&principal_identity)
                .map_err(|denial| {
                    primary_graph_denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::BindingSchemaMismatch,
                        format!("principal identity encoding was rejected: {denial:?}"),
                    )
                })?,
            identity,
            status,
            layout,
        })
    }

    fn admit_principal_row(
        &mut self,
        row: WorthQueryPrincipalBootstrapRow,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let external_identity_key = (row.binding.clone(), row.identity.clone());
        let principal_key = (row.layout.principal_kind, row.principal_key.clone());
        let principal_identity_key = (
            row.binding.clone(),
            authoritative_aspect_value_field_comparison_key(&row.principal_identity),
        );
        if self.external_identities.contains(&external_identity_key) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::DuplicateExternalIdentity,
                &row.binding,
            ));
        }
        if self.principal_keys.contains(&principal_key) || self.entity_keys.contains(&principal_key)
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::DuplicatePrincipalKey,
                &row.binding,
            ));
        }
        if self.principal_identities.contains(&principal_identity_key) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::DuplicatePrincipalIdentity,
                &row.binding,
            ));
        }
        self.external_identities.insert(external_identity_key);
        self.principal_keys.insert(principal_key.clone());
        self.entity_keys.insert(principal_key);
        self.principal_identities.insert(principal_identity_key);
        self.rows.push(row);
        Ok(())
    }

    pub(crate) fn publish(
        self,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial> {
        self.validate_publication_target(runtime, authority)?;
        if self.rows.is_empty() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::EmptyBootstrap,
                "at least one application principal binding is required",
            ));
        }
        let row_count = self.rows.len();
        let entity_count = self.entity_rows.len();
        let relation_count = self.relation_rows.len();
        let principal_identity_index_count = self
            .graph
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.index_id)
            .collect::<BTreeSet<_>>()
            .len();
        let application_equality_index_count = self
            .graph
            .layout
            .equality_index_ids()
            .collect::<BTreeSet<_>>()
            .len();
        let index_ids = self.graph.integration_handle().primary_index_ids.to_vec();
        let commit_id =
            commit_bootstrap_rows(&self.graph, self.rows, self.entity_rows, self.relation_rows)?;
        build_identity_indexes(&self.graph, commit_id, &index_ids)?;
        let binding_identity = self.graph.binding_identity().clone();
        runtime.install_primary_graph(self.graph);
        Ok(WorthQueryPrimaryGraphPublication {
            binding_identity,
            principal_binding_count: row_count,
            identity_index_count: principal_identity_index_count,
            application_equality_index_count,
            policy_entity_count: entity_count,
            policy_relation_count: relation_count,
        })
    }
}

fn primary_graph_denial(
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
