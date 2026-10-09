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

mod advancement_publication;
mod binding_denial;
mod checkpoint;
pub(in crate::domain_computation::primary_graph) mod checkpoint_transition;
mod preparation;
mod program_activation_recovery;
mod program_activation_seeding;
mod publication;
mod publication_target;
mod seed_batches;
mod unique_seed_values;
use binding_denial::map_binding_denial_kind;
mod truth_partition;
pub(super) use program_activation_recovery::recover_program_activation;
use program_activation_seeding::commit_initial_program_activation;
pub(super) use program_activation_seeding::WorthQueryProgramActivationSeed;
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
    pub(super) resource_support: super::provider::WorthQueryPrimaryGraphResourceSupport,
    pub(super) product_world_resources:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    rows: Vec<WorthQueryPrincipalBootstrapRow>,
    external_identities: BTreeSet<(String, WorthQueryExternalPrincipalIdentity)>,
    principal_identities: BTreeSet<(String, AuthoritativeFieldComparisonKey)>,
    unique_seed_values: BTreeSet<unique_seed_values::WorthQueryUniqueSeedValue>,
    principal_keys: BTreeSet<(KindId, String)>,
    pub(super) entity_keys: BTreeSet<(KindId, String)>,
    pub(super) pending_entity_keys: BTreeSet<(KindId, String)>,
    pub(super) relation_keys: BTreeSet<(KindId, String)>,
    pub(super) entity_rows: Vec<super::typed_bootstrap::WorthQueryTypedEntityBootstrapRow>,
    pub(super) relation_rows: Vec<super::typed_bootstrap::WorthQueryTypedRelationBootstrapRow>,
    committed_principal_count: usize,
    committed_entity_count: usize,
    committed_relation_count: usize,
    last_seed_commit_id: Option<worth_relational::facade::history::CommitId>,
    seed_batch_failed: bool,
    pub(super) recovered_publication: Option<WorthQueryPrimaryGraphPublication>,
    recovered_relational_authority:
        Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    recovered_checkpoint_restore_work:
        Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    pub(super) mutation_handlers: super::handler::PendingMutationHandlerRegistry<Schema>,
    /// The initial program this installation activates, seeded before any
    /// ordinary bootstrap row so those rows are validated under its rules.
    pub(super) program_activation_seed: Option<WorthQueryProgramActivationSeed>,
    invariant_installation_receipt:
        worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt,
    expected_invariant_inventory_digest: [u8; 32],
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot{
        self.resource_support.snapshot_ref()
    }
    pub(super) fn take_recovered_relational_authority(
        &mut self,
    ) -> Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority> {
        self.recovered_relational_authority.take()
    }

    pub(super) fn recovered_checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.recovered_checkpoint_restore_work
    }

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
        self.admit_principal_unique_values(&row)?;
        self.external_identities.insert(external_identity_key);
        self.principal_keys.insert(principal_key.clone());
        self.entity_keys.insert(principal_key.clone());
        self.pending_entity_keys.insert(principal_key);
        self.principal_identities.insert(principal_identity_key);
        self.rows.push(row);
        Ok(())
    }

    pub(crate) fn publish(
        self,
        phase: &super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial> {
        self.publish_with_resource_support(phase, runtime, authority)
            .map(|(publication, _resource_support)| publication)
    }
}

fn primary_graph_denial(
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}

pub(super) fn request_denial(
    cause: super::WorthQueryAdvancementDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match cause.provider_denial_cause() {
        Ok(kind) => WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind },
        Err(kind) => WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { kind },
    };
    primary_graph_denial(kind, "bootstrap request admission")
}
