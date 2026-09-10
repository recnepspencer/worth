use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexKind};
use worth_relational::facade::runtime::RelationalRuntime;

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;

#[cfg(test)]
mod test_inspection;

/// Execution-owned primary logical graph for one installed application schema.
///
/// This surface exposes graph identity rather than raw Relational access.
/// Query integration code receives a separate hidden handle so product
/// consumers cannot bypass installed Query authority.
pub struct WorthQueryPrimaryGraph {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    relational_runtime_instance_id: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) layout: Arc<WorthQueryPrimaryGraphLayout>,
    source_owner: WorthQueryRelationalSourceOwner,
    aggregate_projections: Arc<Mutex<super::aggregate_projection::WorthQueryAggregateProjections>>,
    truth_partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
}

impl WorthQueryPrimaryGraph {
    pub(super) fn new(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        mut layout: WorthQueryPrimaryGraphLayout,
        runtime: RelationalRuntime,
    ) -> Self {
        let relational_runtime_instance_id = runtime.main_branch_identity().runtime_instance_id();
        let mut indexes_by_locator = BTreeMap::new();
        for (binding, binding_layout) in layout.principal_bindings_mut() {
            let installed = runtime.index_authority().register(DerivedIndexDefinition {
                index_id: worth_relational::facade::indexes::DerivedIndexId(0),
                name: format!("application-principal.{binding}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: binding_layout.identity_locator.clone(),
                },
                branch_scoped: false,
            });
            binding_layout.index_id = installed.index_id;
            indexes_by_locator.insert(binding_layout.identity_locator.clone(), installed.index_id);
        }
        for ((entity, aspect, field), field_layout) in layout.equality_fields_mut() {
            let index_id = indexes_by_locator
                .get(&field_layout.locator)
                .copied()
                .unwrap_or_else(|| {
                    let installed = runtime.index_authority().register(DerivedIndexDefinition {
                        index_id: worth_relational::facade::indexes::DerivedIndexId(0),
                        name: format!("application-entity.{entity}.{aspect}.{field}"),
                        kind: DerivedIndexKind::EntityField {
                            field_locator: field_layout.locator.clone(),
                        },
                        branch_scoped: false,
                    });
                    indexes_by_locator.insert(field_layout.locator.clone(), installed.index_id);
                    installed.index_id
                });
            field_layout.equality_index_id = Some(index_id);
        }
        layout.register_continuation_orderings(|definition| {
            runtime.index_authority().register(definition).index_id
        });
        layout.register_capability_grant_joins(|definition| {
            runtime.index_authority().register(definition).index_id
        });
        let provider_idempotency = layout.provider_idempotency_mut();
        let installed = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: worth_relational::facade::indexes::DerivedIndexId(0),
            name: "worth-query-provider.idempotency-key".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: provider_idempotency.key_locator.clone(),
            },
            branch_scoped: false,
        });
        provider_idempotency.key_index_id = installed.index_id;
        let aftermath_causality = layout.provider_aftermath_causality_mut();
        let installed = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: worth_relational::facade::indexes::DerivedIndexId(0),
            name: "worth-query-provider.aftermath-causality-key".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aftermath_causality.key_locator.clone(),
            },
            branch_scoped: false,
        });
        aftermath_causality.key_index_id = installed.index_id;
        let source_owner = WorthQueryRelationalSourceOwner::new(runtime, "primary")
            .expect("the installed primary graph role is canonical");
        Self {
            runtime_authority,
            relational_runtime_instance_id,
            binding_identity,
            layout: Arc::new(layout),
            source_owner,
            aggregate_projections: Arc::new(Mutex::new(
                super::aggregate_projection::WorthQueryAggregateProjections::default(),
            )),
            truth_partition_role: None,
        }
    }

    pub(super) fn bind_truth_partition(
        &mut self,
        role: worth_foundational::facade::TruthPartitionRole,
    ) {
        self.source_owner
            .bind_truth_partition(
                "primary",
                worth_relational::facade::identity::PartitionId::main(),
                role.clone(),
            )
            .expect("the installed primary graph partition role is canonical");
        self.truth_partition_role = Some(role);
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub(super) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub(super) const fn relational_runtime_instance_id(&self) -> u64 {
        self.relational_runtime_instance_id
    }

    pub(in crate::domain_computation) fn layout(&self) -> &WorthQueryPrimaryGraphLayout {
        &self.layout
    }

    pub(in crate::domain_computation) fn retain_layout(&self) -> Arc<WorthQueryPrimaryGraphLayout> {
        Arc::clone(&self.layout)
    }

    pub(crate) fn integration_handle(&self) -> WorthQueryPrimaryGraphIntegrationHandle {
        let principal_identity_index_ids = self
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.index_id)
            .collect::<BTreeSet<_>>();
        let primary_index_ids = principal_identity_index_ids
            .iter()
            .copied()
            .chain(self.layout.equality_index_ids())
            .chain(self.layout.continuation_ordering_index_ids())
            .chain(self.layout.capability_grant_join_index_ids())
            .chain(std::iter::once(
                self.layout.provider_idempotency().key_index_id,
            ))
            .chain(std::iter::once(
                self.layout.provider_aftermath_causality().key_index_id,
            ))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .into();
        WorthQueryPrimaryGraphIntegrationHandle {
            source_owner: self.source_owner.clone(),
            layout: Arc::clone(&self.layout),
            primary_index_ids,
            aggregate_projections: Arc::clone(&self.aggregate_projections),
            truth_partition_role: self.truth_partition_role.clone(),
        }
    }

    pub(in crate::domain_computation) fn query_session_port(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort {
        crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort::new(
            self.binding_identity.clone(),
            self.integration_handle(),
        )
    }
}

impl std::fmt::Debug for WorthQueryPrimaryGraph {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryPrimaryGraph")
            .field("binding_identity", &self.binding_identity)
            .finish_non_exhaustive()
    }
}

/// Hidden composition handle for sharing the one primary graph with Query's
/// existing runtime backend.
#[doc(hidden)]
#[derive(Clone)]
pub struct WorthQueryPrimaryGraphIntegrationHandle {
    pub(super) source_owner: WorthQueryRelationalSourceOwner,
    pub(super) layout: Arc<WorthQueryPrimaryGraphLayout>,
    pub(super) primary_index_ids: Arc<[worth_relational::facade::indexes::DerivedIndexId]>,
    pub(super) aggregate_projections:
        Arc<Mutex<super::aggregate_projection::WorthQueryAggregateProjections>>,
    pub(super) truth_partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn with_runtime<T>(&self, read: impl FnOnce(&RelationalRuntime) -> T) -> T {
        self.source_owner.with_runtime(read)
    }

    pub(crate) fn with_runtime_mut<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        self.source_owner.with_runtime_mut(mutate)
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        self.source_owner.with_runtime_mut_unwind_isolated(mutate)
    }

    pub(in crate::domain_computation) fn with_query_runtime_mut<T>(
        &self,
        read: impl FnOnce(&mut RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T,
    ) -> T {
        self.source_owner
            .with_runtime_mut(|runtime| read(runtime, &self.layout))
    }

    /// Retains the shared relational source for a host-owned runtime Bridge
    /// that must observe this exact primary graph.
    ///
    /// The source carries graph access, not invalidation admission authority.
    #[doc(hidden)]
    pub fn relational_bridge_source(
        &self,
    ) -> worth_relational::facade::bridge::RuntimeBridgeRelationalSource {
        self.source_owner.bridge_source()
    }

    #[doc(hidden)]
    pub fn current_truth_snapshot(
        &self,
        branch: &worth_runtime_bridge::facade::TruthBranchIdentity,
    ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.source_owner.current_truth_snapshot(branch)
    }

    pub(crate) fn bind_current_truth_head(
        &self,
        branch: &worth_relational::facade::history::BranchId,
    ) -> Result<
        worth_runtime_bridge::facade::TruthSnapshotIdentity,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.source_owner.bind_current_truth_head(branch)
    }

    pub(crate) fn bind_truth_head_basis_in_runtime(
        &self,
        runtime: &RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    ) -> Result<
        worth_runtime_bridge::facade::TruthSnapshotIdentity,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.source_owner
            .bind_truth_head_basis_in_runtime(runtime, basis)
    }

    /// Admits product installation without exposing a mutation route that could
    /// bypass this primary graph's required index maintenance.
    #[doc(hidden)]
    pub fn prepare_product_source(
        &self,
        branch: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductRelationalInstallation,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    >{
        self.source_owner.prepare_product_source(branch)
    }

    pub(crate) const fn truth_partition_role(
        &self,
    ) -> Option<&worth_foundational::facade::TruthPartitionRole> {
        self.truth_partition_role.as_ref()
    }
}
