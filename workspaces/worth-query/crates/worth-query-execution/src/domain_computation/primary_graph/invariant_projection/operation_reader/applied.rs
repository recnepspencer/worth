//! A producer decision on an actual unpublished image has no Current snapshot receipt.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    RetainedSourceFacts, StorageControl,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
    RelationalPinnedRead,
};

/// Plain decision facts remain qualified by the actual Native proposal and revision.
/// The independently admitted before product supplies authorization only.
pub(in crate::domain_computation::primary_graph) struct AppliedOperationProjection<
    Schema,
    Operation,
> {
    pub(in crate::domain_computation::primary_graph) admission:
        WorthQueryOperationAdmissionIdentity,
    pub(in crate::domain_computation::primary_graph) candidate:
        RelationalAppliedCandidateReadQualification,
    pub(in crate::domain_computation::primary_graph) scope:
        super::super::WorthQueryRealizedProjectionScope,
    pub(in crate::domain_computation::primary_graph) facts: DecisionReads,
    pub(in crate::domain_computation::primary_graph) source_facts: RetainedSourceFacts,
    pub(in crate::domain_computation::primary_graph) consumed_outputs: std::collections::BTreeMap<
        Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,
        super::super::ConsumedOutputEvidence,
    >,
    pub(super) product: crate::basis::WorthQueryProductBranchLease,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(in crate::domain_computation::primary_graph) fn project_applied_operation<
        Operation,
        Input,
        Scope,
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        predecessor_access: Option<
            &crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
                '_,
                Schema,
            >,
        >,
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
        let lease = admission
            .graph_work()
            .mutation_lease()
            .expect("admitted operation retains its genuine before lease");
        let product = lease.product();
        let completed = self
            .read_native(
                execution,
                admission
                    .allowed_graph_contract()
                    .projection_work_budget()
                    .map_or_else(
                        super::super::work::WorthQueryInvariantProjectionWorkBudget::unbounded,
                        super::super::work::WorthQueryInvariantProjectionWorkBudget::bounded,
                    ),
                product.relational_basis(),
                lease.snapshot(),
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
                product: product.retained_clone(),
                marker: PhantomData,
            },
            completed.work,
        ))
    }
}
