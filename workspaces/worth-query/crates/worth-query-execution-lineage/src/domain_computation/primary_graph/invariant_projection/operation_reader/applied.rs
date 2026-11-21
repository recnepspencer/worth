//! A producer decision on an actual unpublished image has no Current snapshot receipt.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    RetainedSourceFacts, StorageControl,
};
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
    RelationalPinnedRead,
};

/// Plain decision facts remain qualified by the actual Native proposal and revision.
/// The independently admitted before product supplies authorization only.
pub struct AppliedOperationProjection<Schema, Operation> {
    pub admission: WorthQueryOperationAdmissionIdentity,
    pub candidate: RelationalAppliedCandidateReadQualification,
    pub scope: super::super::WorthQueryRealizedProjectionScope,
    pub facts: DecisionReads,
    pub source_facts: RetainedSourceFacts,
    pub applied_consumptions:
        Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    pub consumed_outputs: std::collections::BTreeMap<
        Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,
        super::super::ConsumedOutputEvidence,
    >,
    pub product: crate::basis::WorthQueryProductBranchLease,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub fn project_applied_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        predecessor_access: Option<&dyn SourceQueryReaderPort>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            AppliedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: crate::ports::TypedAdmittedOperationPort<Schema>,
    {
        self.project_applied_operation_port(
            execution,
            admission,
            before,
            candidate,
            predecessor_access,
            projection,
            allocation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn project_applied_operation_port<Operation, Scope, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &dyn AdmittedOperationPort,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        predecessor_access: Option<&dyn SourceQueryReaderPort>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            AppliedOperationProjection<Schema, Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    > {
        admission.validate_projection_authority(
            self.runtime_authority,
            &self.binding_identity,
            &self.installed_operations,
        )?;
        let product = admission
            .mutation_product()
            .expect("admitted operation retains its genuine before lease");
        let snapshot = admission
            .mutation_snapshot()
            .expect("admitted operation retains its genuine before lease");
        let completed = self
            .read_native(
                execution,
                admission.projection_work_budget().map_or_else(
                    super::super::work::WorthQueryInvariantProjectionWorkBudget::unbounded,
                    super::super::work::WorthQueryInvariantProjectionWorkBudget::bounded,
                ),
                product.relational_basis(),
                snapshot,
                Some((before, candidate)),
                |reader| {
                    super::admitted::project(
                        self,
                        reader,
                        admission,
                        product,
                        projection,
                        super::admitted::SourceAccess {
                            source_query: predecessor_access,
                            // Nested query/managed-index reads need a genuine
                            // applied query posture. This loan admits only the
                            // sealed predecessor reader, which checks C itself.
                            source_context_use: None,
                        },
                    )
                },
                StorageControl::new(allocation, Some(admission.publication_request()))
                    .with_execution(*execution),
            )
            .map_err(|cause| {
                WorthQueryOperationProjectionDenial::from_invariant(cause, admission.operation())
            })?;
        let (output, facts) = completed.output;
        Ok((
            output,
            AppliedOperationProjection {
                admission: admission.admission_identity(),
                candidate: candidate.candidate_identity().qualification(),
                scope: completed.realized_scope,
                facts,
                source_facts: completed.dependent_source_facts,
                consumed_outputs: completed.consumed_outputs,
                applied_consumptions: completed.applied_consumptions,
                product: product.retained_clone(),
                marker: PhantomData,
            },
            completed.work,
        ))
    }
}
