//! Descriptive extension of one native completed readset, retaining its existing lease.
use super::*;
use crate::{
    basis::WorthQueryProductBranchReadIdentity,
    domain_computation::primary_graph::application_attempt::{
        retained_decision_facts::{RetainedSourceFacts, StorageControl},
        snapshot_lease::WorthQueryApplicationSnapshotLease,
    },
};
use std::collections::BTreeMap;

pub struct BorrowedOperationProjection<
    Schema,
    Operation,
> {
    pub admission:
        WorthQueryOperationAdmissionIdentity,
    pub basis: WorthQueryProductBranchReadIdentity,
    pub scope:
        super::super::WorthQueryRealizedProjectionScope,
    pub facts: DecisionReads,
    pub source_facts: RetainedSourceFacts,
    pub consumed_outputs: BTreeMap<
        Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,
        super::super::ConsumedOutputEvidence,
    >,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub fn project_existing_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        lease: &WorthQueryApplicationSnapshotLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, A::Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            BorrowedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: crate::ports::TypedAdmittedOperationPort<Schema>,
    {
        self.project_existing_operation_port(execution, admission, lease, projection, allocation)
    }

    pub fn project_existing_operation_port<Operation, Scope, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &dyn AdmittedOperationPort,
        lease: &WorthQueryApplicationSnapshotLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            BorrowedOperationProjection<Schema, Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    > {
        admission.validate_projection_authority(
            self.runtime_authority,
            &self.binding_identity,
            &self.installed_operations,
        )?;
        let product = lease.product();
        let completed = self
            .project_borrowed(
                execution,
                lease,
                admission.projection_work_budget(),
                |reader| {
                    super::admitted::project(
                        self,
                        reader,
                        admission,
                        product,
                        projection,
                        super::admitted::SourceAccess::none(),
                    )
                },
                StorageControl::new(allocation, Some(admission.publication_request()))
                    .with_execution(*execution),
            )
            .map_err(|denial| {
                WorthQueryOperationProjectionDenial::from_invariant(denial, admission.operation())
            })?;
        let (output, facts) = completed.output;
        Ok((
            output,
            BorrowedOperationProjection {
                admission: admission.admission_identity(),
                basis: WorthQueryProductBranchReadIdentity::from_observation(product.observation()),
                scope: completed.realized_scope,
                facts,
                source_facts: completed.dependent_source_facts,
                consumed_outputs: completed.consumed_outputs,
                marker: PhantomData,
            },
            completed.work,
        ))
    }
}
