use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationSchema, DeclaredApplicationFieldValue,
    EqualityPredicate, WritePosture,
};

use super::work::WorthQueryInvariantProjectionWorkBudget;
use super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionWork,
    WorthQueryRealizedProjectionScope,
};
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};

pub struct WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    pub(super) runtime: &'runtime mut worth_relational::facade::runtime::RelationalRuntime,
    pub(super) layout: &'runtime super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    pub(super) snapshot: &'runtime worth_relational::facade::snapshots::SnapshotHandle,
    entity_resolution: &'runtime super::super::WorthQueryInstalledEntityResolutionContext,
    pub(super) authority_identity: u64,
    pub(super) work: WorthQueryInvariantProjectionWork,
    pub(super) work_budget: WorthQueryInvariantProjectionWorkBudget,
    pub(super) realized_scope: WorthQueryRealizedProjectionScope,
    pub(super) aggregate_projections:
        Arc<std::sync::Mutex<super::super::aggregate_projection::WorthQueryAggregateProjections>>,
    _schema: PhantomData<fn() -> Schema>,
}

pub struct WorthQueryCompletedInvariantProjection<Schema, Output> {
    output: Output,
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    work: WorthQueryInvariantProjectionWork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantProjectionTraversalDenialKind {
    RelationNotInstalled,
    UndeclaredDecisionTarget,
    ForeignIdentity,
    EndpointUnavailable,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionTraversalDenial {
    kind: WorthQueryInvariantProjectionTraversalDenialKind,
    relation: String,
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn project<Output>(
        &self,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let basis = self.graph.with_runtime_mut(|runtime| {
            let identity = runtime.main_branch_identity();
            runtime
                .observe_branch(&identity)
                .map(|(_, basis)| basis)
                .map_err(super::admission_denial::from_branch_basis_denial)
        })?;
        self.project_with_work_budget(
            WorthQueryInvariantProjectionWorkBudget::unbounded(),
            basis,
            projection,
        )
    }

    pub(super) fn project_bounded<Output>(
        &self,
        maximum_work: usize,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        self.project_with_work_budget(
            WorthQueryInvariantProjectionWorkBudget::bounded(maximum_work),
            basis,
            projection,
        )
    }

    fn project_with_work_budget<Output>(
        &self,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let snapshot = self.graph.with_runtime_mut(|runtime| {
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(super::admission_denial::from_snapshot_admission_denial)
        })?;
        let projected = self.graph.with_runtime_mut(|runtime| {
            catch_unwind(AssertUnwindSafe(|| {
                let mut reader = WorthQueryApplicationInvariantProjectionReader {
                    runtime,
                    layout: &self.layout,
                    snapshot: &snapshot,
                    entity_resolution: &self.entity_resolution,
                    authority_identity: self.authority_identity,
                    work: WorthQueryInvariantProjectionWork::default(),
                    work_budget,
                    realized_scope: WorthQueryRealizedProjectionScope::default(),
                    aggregate_projections: Arc::clone(&self.graph.aggregate_projections),
                    _schema: PhantomData,
                };
                let output = projection(&mut reader);
                (
                    output,
                    reader.work,
                    reader.realized_scope,
                    reader.work_budget.exceeded(),
                )
            }))
        });
        let (output, work, realized_scope, exceeded) = match projected {
            Ok(completed) => completed,
            Err(payload) => {
                self.graph.with_runtime_mut(|runtime| {
                    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                });
                resume_unwind(payload)
            }
        };
        if exceeded {
            self.graph.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
            return Err(WorthQueryInvariantProjectionDenial::work_budget_exceeded());
        }
        Ok(WorthQueryCompletedInvariantProjection {
            output,
            snapshot: WorthQueryApplicationInvariantProjectionSnapshot {
                graph: self.graph.clone(),
                layout: Arc::clone(&self.layout),
                basis: Some(basis),
                snapshot: Some(snapshot),
                runtime_authority: self.runtime_authority,
                binding_identity: self.binding_identity.clone(),
                authority_identity: self.authority_identity,
                realized_scope,
                _schema: PhantomData,
            },
            work,
        })
    }
}

impl<Schema, Output> WorthQueryCompletedInvariantProjection<Schema, Output> {
    pub const fn output(&self) -> &Output {
        &self.output
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    pub fn into_parts(
        self,
    ) -> (
        Output,
        WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
        WorthQueryInvariantProjectionWork,
    ) {
        (self.output, self.snapshot, self.work)
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub const fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.snapshot.version_id()
    }

    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if !self.work_budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                field.field(),
            ));
        }
        let value = Field::Binding::encode(&value).map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            )
        })?;
        let truth = self.entity_resolution.at_snapshot(
            self.runtime,
            self.snapshot,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )?;
        let (resolved, examined) =
            truth.resolve_with_work(field.entity(), field.aspect(), field.field(), value);
        self.work_budget.consume(1 + examined);
        self.work.record_lookup(examined);
        let resolved = resolved?;
        self.realized_scope.record(resolved.entity_id());
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id: resolved.entity_id(),
            kind: resolved.entity_kind(),
            entity: Arc::from(field.entity()),
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        })
    }

    pub fn resolve_optional_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        match self.resolve_entity(field, value) {
            Ok(identity) => Ok(Some(identity)),
            Err(denial) if denial.kind() == WorthQueryEntityResolutionDenialKind::UnknownEntity => {
                Ok(None)
            }
            Err(denial) => Err(denial),
        }
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<Value>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if !self.identity_is_local(identity, field.entity()) {
            return None;
        }
        if !self.work_budget.can_afford(1) {
            return None;
        }
        self.work_budget.consume(1);
        self.realized_scope.record(identity.entity_id);
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())?
            .clone();
        self.work.record_field();
        super::super::application_attempt::observe_field_value(
            self.runtime,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
        )
        .and_then(|value| Field::Binding::decode(&value).ok())
    }

    pub(super) fn identity_is_local<Entity>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
    ) -> bool {
        identity.authority_identity == self.authority_identity && identity.entity.as_ref() == entity
    }
}

impl WorthQueryInvariantProjectionTraversalDenial {
    pub const fn kind(&self) -> WorthQueryInvariantProjectionTraversalDenialKind {
        self.kind
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub(super) fn new(
        kind: WorthQueryInvariantProjectionTraversalDenialKind,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            relation: relation.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryInvariantProjectionTraversalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invariant projection traversal denied: {:?} ({})",
            self.kind, self.relation
        )
    }
}

impl std::error::Error for WorthQueryInvariantProjectionTraversalDenial {}
