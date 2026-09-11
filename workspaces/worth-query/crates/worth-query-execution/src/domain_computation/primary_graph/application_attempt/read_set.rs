use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding, ApplicationSchema,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads,
    WorthQueryOperationGraphReadScope, WritePosture,
};

use super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use super::read_phase::{
    WorthQueryOrdinaryApplicationRead, WorthQueryProjectedApplicationMutation,
};
use super::read_scope::WorthQueryApplicationReadScope;
use super::snapshot_lease::WorthQueryApplicationSnapshotLease;
use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationOperationInvariantProjectionSnapshot,
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};

mod observation_admission;
mod observations;
mod projected_completion;

pub struct WorthQueryApplicationReadAttempt<
    Schema,
    Operation,
    Input,
    Scope,
    Phase = WorthQueryOrdinaryApplicationRead,
> {
    admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: WorthQueryApplicationSnapshotLease,
    layout: Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    entity_resolution: WorthQueryInstalledEntityResolutionContext,
    read_scope: WorthQueryApplicationReadScope,
    expected_facts: Option<BTreeSet<WorthQueryApplicationFactKey>>,
    installed_read_scopes:
        BTreeMap<WorthQueryApplicationFactKey, WorthQueryOperationGraphReadScope>,
    facts: BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    _phase: PhantomData<fn() -> Phase>,
}

pub struct WorthQueryCompleteApplicationReadSet<
    Schema,
    Operation,
    Input,
    Scope,
    Phase = WorthQueryOrdinaryApplicationRead,
> {
    pub(super) admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    pub(super) lease: WorthQueryApplicationSnapshotLease,
    pub(super) installed_read_scopes: Vec<WorthQueryOperationGraphReadScope>,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) _phase: PhantomData<fn() -> Phase>,
}

pub struct WorthQueryObservedApplicationRelation<Schema, Relation, From, To> {
    count: usize,
    pub(super) matching_relations: Vec<worth_relational::facade::identity::RelationId>,
    _marker: PhantomData<fn() -> (Schema, Relation, From, To)>,
}

impl<Schema, Relation, From, To> WorthQueryObservedApplicationRelation<Schema, Relation, From, To> {
    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn is_absent(&self) -> bool {
        self.count == 0
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn begin_application_read_attempt<Operation, Input, Scope>(
        &self,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            ));
        }
        admission.validate_current_authority().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                admission.operation(),
            )
        })?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let read_scope = WorthQueryApplicationReadScope::root_only(admission.scope_entity_id());
        let lease = admission
            .graph_work_mut()
            .take_mutation_lease()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                    admission.operation(),
                )
            })?;
        Ok(WorthQueryApplicationReadAttempt {
            admission,
            lease,
            layout: Arc::clone(&graph.layout),
            entity_resolution: graph.retain_entity_resolution_context(),
            read_scope,
            expected_facts: None,
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            _phase: PhantomData,
        })
    }

    /// Consumes a projection lease typed for this exact admitted operation.
    ///
    /// A same-schema projection for another operation cannot enter the
    /// application read-set progression:
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::{
    ///     WorthQueryAdmittedApplicationOperation,
    ///     WorthQueryApplicationOperationInvariantProjectionSnapshot,
    ///     WorthQueryPrimaryGraphApplicationRuntime,
    /// };
    /// use worth_query_installation::facade::ApplicationSchema;
    ///
    /// struct FirstOperation;
    /// struct SecondOperation;
    ///
    /// fn cannot_cross_operation_projection<Schema: ApplicationSchema, Input, Scope>(
    ///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ///     admission: WorthQueryAdmittedApplicationOperation<
    ///         Schema, FirstOperation, Input, Scope,
    ///     >,
    ///     projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<
    ///         Schema, SecondOperation,
    ///     >,
    /// ) {
    ///     let _ = runtime.begin_projected_application_read_attempt(admission, projection);
    /// }
    /// ```
    pub fn begin_projected_application_read_attempt<Operation, Input, Scope>(
        &self,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            ));
        }
        if !projection.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
            admission.admission_identity(),
        ) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                admission.operation(),
            ));
        }
        admission.validate_current_authority().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                admission.operation(),
            )
        })?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let root = admission.scope_entity_id();
        let (lease, projected_scope, expected_facts) = projection.into_lease_and_realized_scope();
        let layout = Arc::clone(&lease.layout);
        Ok(WorthQueryApplicationReadAttempt {
            admission,
            lease,
            layout,
            entity_resolution: graph.retain_entity_resolution_context(),
            read_scope: WorthQueryApplicationReadScope::projected(root, projected_scope),
            expected_facts: Some(expected_facts),
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            _phase: PhantomData,
        })
    }
}

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<Schema, Operation, Input, Scope, Phase>
{
    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &self,
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
        WorthQueryApplicationEntityIdentity<Schema, Entity>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let value = Field::Binding::encode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
                field.field(),
            )
        })?;
        let resolved = self
            .lease
            .handle()
            .with_runtime(|runtime| {
                self.entity_resolution
                    .at_snapshot(
                        runtime,
                        self.lease.snapshot(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .and_then(|truth| {
                        truth.resolve(field.entity(), field.aspect(), field.field(), value)
                    })
            })
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    field.field(),
                )
            })?;
        if !self.read_scope.admits(resolved.entity_id()) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                field.entity(),
            ));
        }
        Ok(resolved.into_application_identity())
    }

    pub fn complete(
        self,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        if self.facts.len()
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        if self
            .expected_facts
            .as_ref()
            .is_some_and(|expected| !self.facts.keys().eq(expected.iter()))
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        let installed_scopes_close_over_facts = self
            .installed_read_scopes
            .iter()
            .zip(self.facts.keys())
            .all(|((scope_key, scope), fact_key)| {
                scope_key == fact_key
                    && observation_admission::graph_read_scope_matches_key(scope, fact_key)
                    && self
                        .admission
                        .allowed_graph_contract()
                        .graph_reads()
                        .roles()
                        .iter()
                        .flat_map(|role| role.read_scopes())
                        .any(|installed| installed == scope)
            });
        if self.installed_read_scopes.len() != self.facts.len()
            || !installed_scopes_close_over_facts
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        if self.expected_facts.is_none()
            && !observation_admission::graph_reads_exactly_cover_fact_keys(
                self.admission.allowed_graph_contract().graph_reads(),
                self.facts.keys(),
            )
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::IncompleteDecisionReadSet,
                self.admission.operation(),
            ));
        }
        self.admission
            .mutation_preconditions()
            .validate_observations(&self.facts)
            .map_err(|()| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MutationPreconditionMismatch,
                    self.admission.operation(),
                )
            })?;
        Ok(WorthQueryCompleteApplicationReadSet {
            admission: self.admission,
            lease: self.lease,
            installed_read_scopes: self.installed_read_scopes.into_values().collect(),
            facts: self.facts.into_values().collect(),
            _phase: PhantomData,
        })
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
