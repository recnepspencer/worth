use super::*;
impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn begin_application_read_attempt<'request, 'authority, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        admission
            .refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
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
            .and_then(|()| admission.validate_current_authority())
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
        let source_facts =
            SourceFacts::Admitted(validate_source_facts(execution, &mut admission, &lease)?);
        Ok(WorthQueryApplicationReadAttempt {
            execution: *execution,
            value_decode: super::super::super::value_decode::ValueDecodeState::new(self.installed_schema.value_bindings().clone()),
            admission,
            lease,
            layout: Arc::clone(&graph.layout),
            entity_resolution: graph.retain_entity_resolution_context(),
            read_scope,
            expected_facts: None,
            computation_reads: (crate::domain_computation::primary_graph::invariant_projection::ObservedComputationInputs::None, crate::domain_computation::primary_graph::application_contribution::CompletedComputationRetention::Absent(
                crate::domain_computation::primary_graph::application_contribution::PriorAbsence::NotProduced)),
            installed_read_scopes: BTreeMap::new(),
            facts: BTreeMap::new(),
            source_facts,
            consumed_outputs: Vec::new(),
            native_completion: None,
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
    ///     let _ = runtime.begin_projected_application_read_attempt(admission, projection, worth_execution::ExecutionAllocationPolicy::SystemAllocation);
    /// }
    /// ```
    pub fn begin_projected_application_read_attempt<'scope, Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'scope,
            'scope,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let phase_request = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "application-read-phase")),
        };
        let execution = &phase_request;
        self.begin_projected_application_read_attempt_with_request(
            execution,
            admission,
            projection,
            allocation_policy,
        )
    }

    pub(in crate::domain_computation) fn begin_projected_application_read_attempt_with_request<
        'request,
        'authority,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
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
        admission
            .refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
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
        let (lease, projected_scope, decision_reads, dependent_source_facts, consumed_outputs) =
            projection.into_lease_and_realized_scope();
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
