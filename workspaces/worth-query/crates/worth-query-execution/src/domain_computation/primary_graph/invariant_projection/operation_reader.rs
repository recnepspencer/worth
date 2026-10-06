use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, ApplicationSchema, ApplicationSignedAggregateValueBinding,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads,
    WorthQueryOperationGraphReadContract, WritePosture,
};

mod computation_calls;
mod current_output;
mod decision_plan;
mod decision_reads;
mod prior_output;

pub(in crate::domain_computation::primary_graph) use computation_calls::ComputationCallCharge;
pub use current_output::{
    WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind,
    WorthQueryCurrentOutputSelection,
};
pub use decision_plan::{
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
};
pub(in crate::domain_computation::primary_graph) use decision_reads::DecisionReads;
pub use prior_output::WorthQueryPriorOutputFamilyMember;
pub(in crate::domain_computation::primary_graph) use prior_output::WorthQueryPriorOutputRead;

use super::super::request_local::{assert_request_local, RequestLocal};
use super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryCompletedInvariantProjection,
    WorthQueryInvariantAggregateDenial, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantMutationTarget, WorthQueryInvariantProjectionDenial,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionWork,
    WorthQueryInvariantRelation, WorthQueryOperationProjectionDenial,
};
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryEntityResolutionDenial,
};

/// An invariant projection reader restricted to one operation's declared reads.
///
/// Decision reads record the facts the operation's outcome depends on. Only a
/// reader opened for an admitted operation can mint mutation targets.
pub struct WorthQueryApplicationOperationInvariantProjectionReader<
    'reader,
    'runtime,
    Schema,
    Operation,
> {
    reader: &'reader mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>,
    admitted_graph_reads: Option<&'reader WorthQueryOperationGraphReadContract>,
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    binding_identity: &'reader worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    admission_identity: Option<WorthQueryOperationAdmissionIdentity>,
    operation_scope:
        Option<crate::domain_computation::authorization::WorthQueryOperationScopeBinding>,
    decision_facts: &'reader mut DecisionReads,
    _operation: PhantomData<fn() -> Operation>,
    request_local: RequestLocal,
}

assert_request_local!(
    WorthQueryApplicationOperationInvariantProjectionReader<'static, 'static, (), ()>,
);

/// The result of projecting for an admitted operation: the output, the work
/// spent, and a snapshot typed for that operation.
///
/// Pass the snapshot from `into_parts` to
/// `begin_projected_application_read_attempt`.
pub struct WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output> {
    completed: WorthQueryCompletedInvariantProjection<Schema, (Output, DecisionReads)>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    _operation: PhantomData<fn() -> Operation>,
}

/// The result of an operation-typed inspection: the output and the work spent.
///
/// It keeps no snapshot and grants no mutation authority.
pub struct WorthQueryInspectedOperationInvariantProjection<Operation, Output> {
    output: Output,
    work: WorthQueryInvariantProjectionWork,
    _operation: PhantomData<fn() -> Operation>,
}

/// A projection snapshot bound to one admitted operation, carrying the decision
/// facts it read.
///
/// Only a read attempt for that same operation accepts it.
pub struct WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation> {
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    decision_facts: DecisionReads,
    _operation: PhantomData<fn() -> Operation>,
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    /// Runs an operation-typed inspection without retaining mutation authority.
    ///
    /// The inspection result has no snapshot extraction transition:
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::
    ///     WorthQueryApplicationInvariantProjectionAuthority;
    /// use worth_query_installation::facade::ApplicationSchema;
    ///
    /// struct Operation;
    ///
    /// fn inspection_cannot_become_projection<Schema: ApplicationSchema>(
    ///     authority: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    /// ) {
    ///     let inspected = authority.project_operation::<Operation, _>(|_| ());
    ///     let _ = inspected.into_parts();
    /// }
    /// ```
    pub fn project_operation<Operation, Output>(
        &self,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let completed = self.project(|reader| {
            let mut decision_facts = DecisionReads::default();
            let mut operation_reader = WorthQueryApplicationOperationInvariantProjectionReader {
                reader,
                admitted_graph_reads: None,
                runtime_authority: self.runtime_authority,
                binding_identity: &self.binding_identity,
                admission_identity: None,
                operation_scope: None,
                decision_facts: &mut decision_facts,
                _operation: PhantomData,
                request_local: PhantomData,
            };
            let output = projection(&mut operation_reader);
            (output, decision_facts)
        })?;
        let ((output, _), snapshot, work) = completed.into_parts();
        drop(snapshot);
        Ok(WorthQueryInspectedOperationInvariantProjection {
            output,
            work,
            _operation: PhantomData,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn project_operation_on_product<
        Operation,
        Output,
    >(
        &self,
        product: &crate::basis::WorthQueryProductBranchLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let completed =
            self.project_bounded(usize::MAX, product.relational_basis().clone(), |reader| {
                let mut decision_facts = DecisionReads::default();
                let mut operation_reader =
                    WorthQueryApplicationOperationInvariantProjectionReader {
                        reader,
                        admitted_graph_reads: None,
                        runtime_authority: self.runtime_authority,
                        binding_identity: &self.binding_identity,
                        admission_identity: None,
                        operation_scope: None,
                        decision_facts: &mut decision_facts,
                        _operation: PhantomData,
                        request_local: PhantomData,
                    };
                let output = projection(&mut operation_reader);
                (output, decision_facts)
            })?;
        let ((output, _), snapshot, work) = completed.into_parts();
        drop(snapshot);
        Ok(WorthQueryInspectedOperationInvariantProjection {
            output,
            work,
            _operation: PhantomData,
        })
    }

    pub fn project_admitted_operation<Operation, Input, Scope, Output>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>,
        WorthQueryOperationProjectionDenial,
    > {
        admission.validate_projection_authority(self.runtime_authority, &self.binding_identity)?;
        let product = admission
            .graph_work()
            .mutation_lease()
            .expect("admitted operation retains its selected mutation lease")
            .product()
            .retained_clone();
        let completed = self
            .project_bounded(
                admission.allowed_graph_contract().projection_work_budget(),
                product.relational_basis().clone(),
                |reader| {
                    reader.selected_product_occurrence =
                        Some(product.observation().lifecycle_incarnation());
                    reader.selected_product_generation =
                        Some(product.observation().reference_generation().get());
                    reader.selected_source_partition_identity =
                        admission.source_partition_identity();
                    let mut decision_facts = admission
                        .computation_prior()
                        .map_or_else(DecisionReads::default, DecisionReads::with_prior);
                    let mut operation_reader =
                        WorthQueryApplicationOperationInvariantProjectionReader {
                            reader,
                            admitted_graph_reads: Some(
                                admission.allowed_graph_contract().graph_reads(),
                            ),
                            runtime_authority: self.runtime_authority,
                            binding_identity: &self.binding_identity,
                            admission_identity: Some(admission.admission_identity()),
                            operation_scope: Some(admission.operation_scope_binding().clone()),
                            decision_facts: &mut decision_facts,
                            _operation: PhantomData,
                            request_local: PhantomData,
                        };
                    operation_reader
                        .reader
                        .realized_scope
                        .record(admission.scope_entity_id());
                    let scope = WorthQueryInvariantEntityIdentity {
                        entity_id: admission.scope_entity_id(),
                        kind: admission.scope_entity_kind(),
                        entity: Arc::from(admission.scope_entity_name()),
                        authority_identity: operation_reader.reader.authority_identity,
                        _marker: PhantomData,
                    };
                    let output = projection(&mut operation_reader, &scope);
                    (output, decision_facts)
                },
            )
            .map_err(|denial| {
                WorthQueryOperationProjectionDenial::from_invariant(denial, admission.operation())
            })?;
        Ok(WorthQueryCompletedOperationInvariantProjection {
            completed,
            admission_identity: admission.admission_identity(),
            product,
            _operation: PhantomData,
        })
    }
}

impl<Operation, Output> WorthQueryInspectedOperationInvariantProjection<Operation, Output> {
    pub const fn output(&self) -> &Output {
        &self.output
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    pub fn into_output(self) -> Output {
        self.output
    }
}

impl<Schema, Operation, Output>
    WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>
{
    pub const fn output(&self) -> &Output {
        &self.completed.output().0
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.completed.work()
    }

    pub fn into_parts(
        self,
    ) -> (
        Output,
        WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        WorthQueryInvariantProjectionWork,
    ) {
        let ((output, decision_facts), snapshot, work) = self.completed.into_parts();
        (
            output,
            WorthQueryApplicationOperationInvariantProjectionSnapshot {
                snapshot,
                admission_identity: self.admission_identity,
                product: self.product,
                decision_facts,
                _operation: PhantomData,
            },
            work,
        )
    }
}

#[path = "operation_reader/read_access.rs"]
mod read_access;

impl<Schema, Operation> WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.snapshot.version()
    }

    pub(in crate::domain_computation::primary_graph) fn belongs_to(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        admission_identity: WorthQueryOperationAdmissionIdentity,
    ) -> bool {
        self.snapshot
            .belongs_to(runtime_authority, binding_identity)
            && self.admission_identity == admission_identity
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease_and_realized_scope(
        self,
    ) -> (
        super::super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        super::WorthQueryRealizedProjectionScope,
        DecisionReads,
        Vec<super::super::application_attempt::WorthQueryApplicationObservedFact>,
        Vec<super::ConsumedOutputEvidence>,
    ) {
        let (lease, scope, dependent_source_facts, consumed_outputs) =
            self.snapshot.into_lease_and_realized_scope(self.product);
        (
            lease,
            scope,
            self.decision_facts,
            dependent_source_facts,
            consumed_outputs,
        )
    }
}
