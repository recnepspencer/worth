mod admission_denial;
mod aggregate;
mod locked_reader;
mod operation_projection_denial;
mod operation_reader;
mod realized_scope;
mod traversal;
mod work;

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationReadableScalarValueBinding, ApplicationRelationRef, ApplicationSchema,
    ApplicationSchemaBindingIdentity, DeclaredApplicationFieldValue, WritePosture,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId, VersionId};
use worth_relational::facade::storage::RecordLifecycleState;

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::{
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphIntegrationHandle,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

pub use admission_denial::{
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionDenialKind,
};
pub use aggregate::{
    WorthQueryInvariantAggregate, WorthQueryInvariantAggregateDenial,
    WorthQueryInvariantAggregateDenialKind,
};
pub use locked_reader::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryCompletedInvariantProjection,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
};
pub use operation_projection_denial::{
    WorthQueryOperationProjectionDenial, WorthQueryOperationProjectionDenialKind,
};
pub use operation_reader::{
    WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryApplicationOperationInvariantProjectionSnapshot,
    WorthQueryCompletedOperationInvariantProjection,
    WorthQueryInspectedOperationInvariantProjection, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantDecisionPlanDenialKind,
};
pub(in crate::domain_computation::primary_graph) use realized_scope::WorthQueryRealizedProjectionScope;
pub use work::WorthQueryInvariantProjectionWork;

static NEXT_INVARIANT_PROJECTION_AUTHORITY: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

/// Installation-minted authority for domain invariant projection.
///
/// The application composition root may retain this while it owns the
/// move-only primary-graph bootstrap. Ordinary consumers cannot construct it
/// from runtime identities or application admissions.
pub struct WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    entity_resolution: WorthQueryInstalledEntityResolutionContext,
    authority_identity: u64,
    _schema: PhantomData<fn() -> Schema>,
}

pub struct WorthQueryApplicationInvariantProjectionSnapshot<Schema> {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
    basis: Option<worth_relational::facade::branch::AdmittedRelationalBranchBasis>,
    snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    authority_identity: u64,
    realized_scope: WorthQueryRealizedProjectionScope,
    _schema: PhantomData<fn() -> Schema>,
}

#[derive(Clone, Debug)]
pub struct WorthQueryInvariantEntityIdentity<Schema, Entity> {
    entity_id: EntityId,
    kind: KindId,
    entity: Arc<str>,
    authority_identity: u64,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// An exact projected entity admitted for the later application effect phase.
/// It is minted only by the operation-typed invariant reader after validating
/// the entity against that reader's authority.
pub struct WorthQueryInvariantMutationTarget<Schema, Entity> {
    pub(in crate::domain_computation::primary_graph) entity_id: EntityId,
    pub(in crate::domain_computation::primary_graph) entity: Arc<str>,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantRelation<Schema, Relation, From, To> {
    relation_id: RelationId,
    from: WorthQueryInvariantEntityIdentity<Schema, From>,
    to: WorthQueryInvariantEntityIdentity<Schema, To>,
    _relation: PhantomData<fn() -> Relation>,
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        WorthQueryApplicationInvariantProjectionAuthority {
            graph: self.graph.integration_handle(),
            layout: Arc::clone(&self.graph.layout),
            runtime_authority: self.runtime_authority,
            binding_identity: self.graph.binding_identity().clone(),
            entity_resolution: self.graph.retain_entity_resolution_context(),
            authority_identity: NEXT_INVARIANT_PROJECTION_AUTHORITY
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _schema: PhantomData,
        }
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn belongs_to_installation(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.binding_identity == binding_identity
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn active_snapshot_count(&self) -> usize {
        self.graph
            .with_runtime_mut(|runtime| runtime.retention().inspect_plan().active_snapshot_count)
    }

    pub fn snapshot(
        &self,
    ) -> Result<
        WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
        WorthQueryInvariantProjectionDenial,
    > {
        let (basis, snapshot) = self.graph.with_runtime_mut(|runtime| {
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime
                .observe_branch(&identity)
                .map_err(admission_denial::from_branch_basis_denial)?;
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(admission_denial::from_snapshot_admission_denial)?;
            Ok((basis, snapshot))
        })?;
        Ok(WorthQueryApplicationInvariantProjectionSnapshot {
            graph: self.graph.clone(),
            layout: Arc::clone(&self.layout),
            basis: Some(basis),
            snapshot: Some(snapshot),
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            authority_identity: self.authority_identity,
            realized_scope: WorthQueryRealizedProjectionScope::default(),
            _schema: PhantomData,
        })
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn version(&self) -> VersionId {
        self.snapshot().version_id()
    }

    pub fn entities<Entity>(
        &self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>> {
        let Some(kind) = self.layout.entity_kind(entity.name()) else {
            return Vec::new();
        };
        self.graph.with_runtime(|runtime| {
            runtime
                .read_truth()
                .visible_entities_of_kind(kind, self.snapshot().version_id())
                .into_iter()
                .filter(|record| record.lifecycle == RecordLifecycleState::Live)
                .map(|record| WorthQueryInvariantEntityIdentity {
                    entity_id: record.entity_id,
                    kind,
                    entity: Arc::from(entity.name()),
                    authority_identity: self.authority_identity,
                    _marker: PhantomData,
                })
                .collect()
        })
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<Value>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if identity.authority_identity != self.authority_identity
            || identity.entity.as_ref() != field.entity()
        {
            return None;
        }
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())?
            .clone();
        self.graph
            .with_runtime(|runtime| {
                super::application_attempt::observe_field_value(
                    runtime,
                    self.snapshot(),
                    identity.entity_id,
                    identity.kind,
                    &locator,
                )
            })
            .and_then(|value| Field::Binding::decode(&value).ok())
    }

    pub fn relations<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>> {
        let Some(layout) = self.layout.relation(relation.name()).cloned() else {
            return Vec::new();
        };
        self.graph.with_runtime(|runtime| {
            runtime
                .read_truth()
                .visible_relations_of_kind(layout.kind, self.snapshot().version_id())
                .into_iter()
                .filter(|record| record.lifecycle == RecordLifecycleState::Live)
                .map(|record| WorthQueryInvariantRelation {
                    relation_id: record.relation_id,
                    from: WorthQueryInvariantEntityIdentity {
                        entity_id: record.source,
                        kind: layout.from,
                        entity: Arc::from(relation.from()),
                        authority_identity: self.authority_identity,
                        _marker: PhantomData,
                    },
                    to: WorthQueryInvariantEntityIdentity {
                        entity_id: record.target,
                        kind: layout.to,
                        entity: Arc::from(relation.to()),
                        authority_identity: self.authority_identity,
                        _marker: PhantomData,
                    },
                    _relation: PhantomData,
                })
                .collect()
        })
    }

    pub(in crate::domain_computation::primary_graph) fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.binding_identity == binding_identity
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease(
        mut self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease {
        let basis = self
            .basis
            .take()
            .expect("projection snapshot retains its owner-admitted basis");
        let snapshot = self
            .snapshot
            .take()
            .expect("projection snapshot remains live until consumed");
        super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease::from_existing(
            self.graph.clone(),
            Arc::clone(&self.layout),
            basis,
            snapshot,
            product,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease_and_realized_scope(
        mut self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> (
        super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
    ) {
        let realized_scope = std::mem::take(&mut self.realized_scope);
        (self.into_lease(product), realized_scope)
    }

    fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
            .as_ref()
            .expect("projection snapshot remains live until consumed")
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    pub fn entity_name(&self) -> &str {
        &self.entity
    }
}

impl<Schema, Entity> PartialEq for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.authority_identity == other.authority_identity
            && self.entity_id == other.entity_id
            && self.kind == other.kind
            && self.entity == other.entity
    }
}

impl<Schema, Entity> Eq for WorthQueryInvariantEntityIdentity<Schema, Entity> {}

impl<Schema, Entity> PartialOrd for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema, Entity> Ord for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.authority_identity,
            self.entity_id,
            self.kind,
            self.entity.as_ref(),
        )
            .cmp(&(
                other.authority_identity,
                other.entity_id,
                other.kind,
                other.entity.as_ref(),
            ))
    }
}

impl<Schema, Relation, From, To> WorthQueryInvariantRelation<Schema, Relation, From, To> {
    pub const fn from(&self) -> &WorthQueryInvariantEntityIdentity<Schema, From> {
        &self.from
    }

    pub const fn to(&self) -> &WorthQueryInvariantEntityIdentity<Schema, To> {
        &self.to
    }

    pub const fn relation_id(&self) -> RelationId {
        self.relation_id
    }
}

impl<Schema> Drop for WorthQueryApplicationInvariantProjectionSnapshot<Schema> {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.graph.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
        }
    }
}
