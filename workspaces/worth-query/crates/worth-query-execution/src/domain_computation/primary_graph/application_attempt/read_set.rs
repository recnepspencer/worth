use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;
use worth_execution::ExecutionArray;

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

mod admitted_arrays;
mod binding_proof;
mod completion;
mod retained_facts;
pub(in crate::domain_computation::primary_graph::application_attempt) use admitted_arrays::admit_array;
mod decision_reuse;
pub(in crate::domain_computation::primary_graph) use decision_reuse::{
    CompletedDecisionReuseProof, PreparedDecisionReuseContext,
};
mod handler_fact_boundary;
pub(in crate::domain_computation) use handler_fact_boundary::CompletedHandlerFactBoundary;
mod observation_admission;
mod observations;
mod projected_completion;
mod relation_observation;
mod source_facts;

pub(super) use binding_proof::{MutationHandlerBindingProof, WorkflowOperationBindingProof};
pub use relation_observation::WorthQueryObservedApplicationRelation;
use source_facts::{merge_source_facts, validate_source_facts, SourceFacts};

/// An in-progress decision read for one admitted operation, begun on a leased
/// snapshot of the branch.
///
/// Get one from the application runtime's `begin_application_read_attempt` (or
/// its projected form). Observe the entities, fields, and relations the operation
/// declares it reads, then call `complete` to seal them into a
/// [`WorthQueryCompleteApplicationReadSet`]. Every read is checked against the
/// operation's installed reads and its admitted scope.
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
    source_facts: SourceFacts,
    consumed_outputs: Vec<super::super::invariant_projection::ConsumedOutputEvidence>,
    _phase: PhantomData<fn() -> Phase>,
}

/// The sealed decision read set of one admitted operation: exactly the facts its
/// installed reads cover, observed on one snapshot.
///
/// Sealing checked the fact budget, that the facts cover the declared reads, and
/// the operation's mutation preconditions. In the projected-mutation phase it
/// begins the effect program that authors the candidate; an ordinary read-phase
/// set cannot author effects.
pub struct WorthQueryCompleteApplicationReadSet<
    Schema,
    Operation,
    Input,
    Scope,
    Phase = WorthQueryOrdinaryApplicationRead,
> {
    pub(super) admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    pub(super) lease: WorthQueryApplicationSnapshotLease,
    pub(super) installed_read_scopes: ExecutionArray<WorthQueryOperationGraphReadScope>,
    pub(super) facts: ExecutionArray<WorthQueryApplicationObservedFact>,
    pub(super) consumed_outputs: Vec<super::super::invariant_projection::ConsumedOutputEvidence>,
    pub(super) workflow_authority_binding: Option<WorkflowOperationBindingProof>,
    pub(super) mutation_handler_binding: Option<MutationHandlerBindingProof>,
    /// The Unix-epoch millisecond the workflow instance this attempt steps
    /// must commit before; the provider checks it again at commit.
    pub(super) workflow_deadline: Option<u64>,
    /// A refusal for a new commit only: a retry of a recorded outcome replays.
    pub(super) new_commit_refusal: Option<WorthQueryApplicationAttemptDenial>,
    pub(super) _phase: PhantomData<fn() -> Phase>,
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
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
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
        let source_facts = SourceFacts::Admitted(validate_source_facts(&mut admission, &lease)?);
        Ok(WorthQueryApplicationReadAttempt {
            admission,
            lease,
            layout: Arc::clone(&graph.layout),
            entity_resolution: graph.retain_entity_resolution_context(),
            read_scope,
            expected_facts: None,
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs: Vec::new(),
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
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                admission.operation(),
            )
        })?;
        let root = admission.scope_entity_id();
        let (lease, projected_scope, expected_facts, dependent_source_facts, consumed_outputs) =
            projection.into_lease_and_realized_scope();
        let mut admission = admission;
        let source_facts = merge_source_facts(
            validate_source_facts(&mut admission, &lease)?,
            dependent_source_facts,
            admission.operation(),
        )?;
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
            source_facts,
            consumed_outputs,
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
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
