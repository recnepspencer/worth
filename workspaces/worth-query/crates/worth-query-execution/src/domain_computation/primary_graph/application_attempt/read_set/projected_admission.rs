//! One admission and completion route for committed and applied projection data.
use super::super::retained_decision_facts::RetainedSourceFacts;
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, DecisionReads, WorthQueryRealizedProjectionScope,
};

pub(super) struct ProjectedReadParts {
    lease: WorthQueryApplicationSnapshotLease,
    projected_scope: WorthQueryRealizedProjectionScope,
    decision_reads: DecisionReads,
    dependent_source_facts: Option<RetainedSourceFacts>,
    consumed_outputs: Vec<ConsumedOutputEvidence>,
}
impl ProjectedReadParts {
    pub(super) fn from_projection<Schema: ApplicationSchema, Operation>(
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
    ) -> Self {
        let (lease, projected_scope, decision_reads, dependent_source_facts, consumed_outputs) =
            projection.into_lease_and_realized_scope();
        Self {
            lease,
            projected_scope,
            decision_reads,
            dependent_source_facts,
            consumed_outputs,
        }
    }
}
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Consumes the genuine before authorization lease and the separately
    /// sealed applied decision. No committed projection snapshot is constructed.
    pub(in crate::domain_computation::primary_graph) fn begin_applied_application_read_attempt<
        'request,
        'authority,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: crate::domain_computation::primary_graph::invariant_projection::AppliedOperationProjection<Schema, Operation>,
        candidate: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let mismatch = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                admission.operation(),
            )
        };
        let lease = admission
            .graph_work()
            .mutation_lease()
            .ok_or_else(mismatch)?;
        if projection.admission != admission.admission_identity()
            || projection.candidate != candidate.candidate_identity().qualification()
            || candidate
                .candidate_identity()
                .proposal()
                .branch_observation()
                != lease.product().relational_basis().reference()
        {
            return Err(mismatch());
        }
        let request = admission.publication_request().clone();
        let lease = admission
            .graph_work_mut()
            .share_mutation_lease(
                super::super::retained_decision_facts::StorageControl::new(
                    allocation_policy,
                    Some(&request),
                )
                .with_execution(*execution),
            )
            .map_err(|cause| cause.into_attempt_denial(admission.operation()))?
            .expect("checked genuine before lease")
            .with_applied_projection(candidate);
        let parts = ProjectedReadParts {
            lease,
            projected_scope: projection.scope,
            decision_reads: projection.facts,
            dependent_source_facts: Some(projection.source_facts),
            consumed_outputs: projection.consumed_outputs.into_values().collect(),
        };
        self.begin_projected_read_parts(execution, admission, parts, allocation_policy)
    }

    pub(super) fn begin_projected_read_parts<'request, 'authority, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        parts: ProjectedReadParts,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
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
        admission
            .validate_installed_operation(&self.runtime.installed_operations)
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
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
        let ProjectedReadParts {
            lease,
            projected_scope,
            decision_reads,
            dependent_source_facts,
            consumed_outputs,
        } = parts;
        // A read whose failure the decision swallowed left an outcome its fact
        // does not hold, so no fact can stand for it.
        let (expected_facts, computation_reads) = decision_reads
            .into_expected(execution)
            .map_err(|failed| failed.into_attempt_denial(admission.operation()))?;
        let mut admission = admission;
        let admitted_source_facts = validate_source_facts(execution, &mut admission, &lease)?;
        let source_facts = merge_source_facts(
            admitted_source_facts,
            dependent_source_facts,
            admission.operation(),
            super::super::retained_decision_facts::StorageControl::new(
                allocation_policy,
                Some(admission.publication_request()),
            )
            .with_execution(*execution),
        )?;
        let layout = Arc::clone(&lease.layout);
        Ok(WorthQueryApplicationReadAttempt {
            execution: *execution,
            value_decode: super::super::super::value_decode::ValueDecodeState::new(
                self.installed_schema.value_bindings().clone(),
            ),
            admission,
            lease,
            layout,
            entity_resolution: graph.retain_entity_resolution_context(),
            read_scope: WorthQueryApplicationReadScope::projected(root, projected_scope),
            expected_facts: Some(expected_facts),
            computation_reads,
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs,
            native_completion: None,
            _phase: PhantomData,
        })
    }
}
