use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_collect_assessment<Spec, Program>(
        self,
        workflow: &'application WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        node_path: impl Into<String>,
    ) -> Result<
        WorthQueryWorkflowAdvanceRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            Program,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowAdvancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
    {
        self.prepare_workflow_request(
            workflow,
            instance,
            WorkflowRequestedAction::CollectAssessment {
                node_path: node_path.into(),
            },
        )
    }
}

impl<'application, 'principal, 'scope, Schema, Spec, Program, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Program,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program:
        worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn into_assessment_demand<Demand>(
        self,
        demand: Demand,
    ) -> Result<
        super::super::WorthQueryWorkflowAssessmentDemandRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            Program,
            Demand,
        >,
        super::super::WorthQueryWorkflowAssessmentDemandPreparationDenial,
    >
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>,
    {
        let required = match self.prepared {
            PreparedWorkflowAdvance::AwaitingAssessment(prepared) => prepared.into_required(),
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingCondition(_)
            | PreparedWorkflowAdvance::AwaitingOperation(_)
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => return Err(
                super::super::WorthQueryWorkflowAssessmentDemandPreparationDenial::not_assessment(),
            ),
        };
        super::super::WorthQueryWorkflowAssessmentDemandRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            self.workflow,
            required,
            demand,
        )
    }

    pub fn into_assessment_demand_for<Demand>(
        self,
        expected: &RequiredWorkflowAssessment,
        demand: Demand,
    ) -> Result<
        super::super::WorthQueryWorkflowAssessmentDemandRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            Program,
            Demand,
        >,
        super::super::WorthQueryWorkflowAssessmentDemandPreparationDenial,
    >
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>,
    {
        let request = self.into_assessment_demand(demand)?;
        if request.required() != expected {
            return Err(
                super::super::WorthQueryWorkflowAssessmentDemandPreparationDenial::requirement_mismatch(),
            );
        }
        Ok(request)
    }

    pub fn accept_assessment<Query>(
        self,
        settlement: &super::super::WorthQueryWorkflowAssessmentDemandSettlement<Query>,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAssessmentAcceptanceDenial> {
        if WorthQueryWorkflowAdvanceAdapter::requested_instance(&self.prepared)
            != settlement.required().instance()
        {
            return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch);
        }
        if let Some(replayed) = WorthQueryWorkflowAdvanceAdapter::resolve_assessment_replay(
            self.application,
            &self.prepared,
            settlement.required(),
            settlement.owner_settlement().retained(),
            settlement.owner_settlement().observed_source(),
            settlement.posture(),
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowAssessmentAcceptanceDenial::Replay)?
        {
            return Ok(replayed);
        }
        let prepared = match self.prepared {
            PreparedWorkflowAdvance::AwaitingAssessment(prepared) => prepared,
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingCondition(_)
            | PreparedWorkflowAdvance::AwaitingOperation(_)
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::NotAwaitingAssessment)
            }
        };
        if !same_requirement(prepared.required(), settlement.required()) {
            return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch);
        }
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit_assessment(
            self.application,
            prepared,
            settlement.owner_settlement().retained(),
            settlement.owner_settlement().observed_source(),
            settlement.posture(),
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowAssessmentAcceptanceDenial::Attempt)
    }
}
