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

pub(in crate::domain_computation::primary_graph) struct BorrowedOperationProjection<
    Schema,
    Operation,
> {
    pub(in crate::domain_computation::primary_graph) admission:
        WorthQueryOperationAdmissionIdentity,
    pub(in crate::domain_computation::primary_graph) basis: WorthQueryProductBranchReadIdentity,
    pub(in crate::domain_computation::primary_graph) scope:
        super::super::WorthQueryRealizedProjectionScope,
    pub(in crate::domain_computation::primary_graph) facts: DecisionReads,
    pub(in crate::domain_computation::primary_graph) source_facts: RetainedSourceFacts,
    pub(in crate::domain_computation::primary_graph) consumed_outputs: BTreeMap<
        Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,
        super::super::ConsumedOutputEvidence,
    >,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(in crate::domain_computation::primary_graph) fn project_existing_operation<
        Operation,
        Input,
        Scope,
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
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
        admission.validate_projection_authority(self.runtime_authority, &self.binding_identity)?;
        let product = lease.product();
        let completed = self
            .project_borrowed(
                execution,
                lease,
                admission.allowed_graph_contract().projection_work_budget(),
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
                StorageControl::new(allocation, Some(admission.publication_request())),
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
