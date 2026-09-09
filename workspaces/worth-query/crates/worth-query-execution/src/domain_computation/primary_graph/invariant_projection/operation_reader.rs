use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef, ApplicationSchema,
    EqualityPredicate, OperationReads, TypedApplicationReadableValue, TypedApplicationValue,
    WorthQueryOperationGraphReadContract, WritePosture,
};

mod decision_plan;

pub use decision_plan::{
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
};

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
    application_attempt::WorthQueryApplicationFactKey, WorthQueryAdmittedApplicationOperation,
    WorthQueryEntityResolutionDenial,
};

pub struct WorthQueryApplicationOperationInvariantProjectionReader<
    'reader,
    'runtime,
    Schema,
    Operation,
> {
    reader: &'reader mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>,
    admitted_graph_reads: Option<&'reader WorthQueryOperationGraphReadContract>,
    decision_facts: &'reader mut BTreeSet<WorthQueryApplicationFactKey>,
    _operation: PhantomData<fn() -> Operation>,
}

pub struct WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output> {
    completed: WorthQueryCompletedInvariantProjection<
        Schema,
        (Output, BTreeSet<WorthQueryApplicationFactKey>),
    >,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    _operation: PhantomData<fn() -> Operation>,
}

pub struct WorthQueryInspectedOperationInvariantProjection<Operation, Output> {
    output: Output,
    work: WorthQueryInvariantProjectionWork,
    _operation: PhantomData<fn() -> Operation>,
}

pub struct WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation> {
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    decision_facts: BTreeSet<WorthQueryApplicationFactKey>,
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
            let mut decision_facts = BTreeSet::new();
            let mut operation_reader = WorthQueryApplicationOperationInvariantProjectionReader {
                reader,
                admitted_graph_reads: None,
                decision_facts: &mut decision_facts,
                _operation: PhantomData,
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
                let mut decision_facts = BTreeSet::new();
                let mut operation_reader =
                    WorthQueryApplicationOperationInvariantProjectionReader {
                        reader,
                        admitted_graph_reads: None,
                        decision_facts: &mut decision_facts,
                        _operation: PhantomData,
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
                    let mut decision_facts = BTreeSet::new();
                    let mut operation_reader =
                        WorthQueryApplicationOperationInvariantProjectionReader {
                            reader,
                            admitted_graph_reads: Some(
                                admission.allowed_graph_contract().graph_reads(),
                            ),
                            decision_facts: &mut decision_facts,
                            _operation: PhantomData,
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
        BTreeSet<WorthQueryApplicationFactKey>,
    ) {
        let (lease, scope) = self.snapshot.into_lease_and_realized_scope(self.product);
        (lease, scope, self.decision_facts)
    }
}
