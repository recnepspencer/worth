//! Real installation and native workflow history for the owning action program.
use super::{model, program::*};
use model::{schema::DocumentRetentionSchema, workflow::*};
use worth_query_host::facade::{
    application_entry::*, application_installation::*, declaration::application_program::*,
};
pub type AssessmentOwner = WorthQueryWorkflowAssessmentOperation<
    DocumentRetentionSchema,
    ReviewedDocumentWorkflow,
    model::assessment_output::RetentionAssessmentDemand,
    WorkflowOwnerProgram,
>;
pub struct Host {
    pub workflow: model::host::DocumentWorkflowRuntime<WorkflowOwnerProgram>,
    pub projection:
        worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
            DocumentRetentionSchema,
        >,
}
pub fn install() -> Host {
    let (workflow, projection) = model::host::publish_owner_workflow_with_projection();
    Host {
        workflow,
        projection,
    }
}

impl Host {
    pub fn begin(
        &self,
        definition: ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
        retention: u64,
        key: u64,
    ) -> PublishedWorkflowInstanceRef {
        self.begin_with_proposal(definition, retention, key).0
    }
    pub fn begin_with_proposal(
        &self,
        definition: ValidatedWorkflowDefinition<ReviewedDocumentWorkflow>,
        retention: u64,
        key: u64,
    ) -> (PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef) {
        let runtime = self.workflow.runtime();
        let scope = model::operator_identity::request_scope();
        let principal =
            model::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        let contract = self
            .workflow
            .workflow_spec()
            .bind_definition(definition)
            .unwrap();
        let published = runtime
            .request(&principal, &scope)
            .mutate(WorkflowDefinitionAuthoringIntent {
                input: WorkflowDefinitionAuthoringInput {
                    identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                    retention_days: retention,
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
            .unwrap()
            .execute();
        let WorkflowDefinitionPublicationOutcome::Published(published) = published else {
            panic!("real definition publication")
        };
        let next = key + 1;
        let started = runtime
            .request(&principal, &scope)
            .mutate(WorkflowInstanceStartIntent {
                input: WorkflowInstanceStartInput {
                    document_identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                },
            })
            .without_source()
            .idempotency(&next)
            .prepare_workflow_instance_start(&self.workflow, published.definition().clone())
            .unwrap()
            .execute();
        let WorkflowInstanceStartOutcome::Started(started) = started else {
            panic!("real instance start")
        };
        let instance = started.instance().clone();
        let next = key + 2;
        let proposal = runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(WorkflowDefinitionAuthoringIntent {
                input: WorkflowDefinitionAuthoringInput {
                    identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                    retention_days: retention,
                },
            })
            .without_source()
            .idempotency(&next)
            .prepare_workflow_proposal(&self.workflow, instance.clone())
            .unwrap()
            .execute();
        let WorkflowProposalOutcome::Published(proposal) = proposal else {
            panic!("real proposal publication")
        };
        (instance, proposal.proposal().clone())
    }
    pub fn advance(
        &self,
        instance: PublishedWorkflowInstanceRef,
        key: u64,
    ) -> WorkflowProgressOutcome {
        let runtime = self.workflow.runtime();
        let scope = model::operator_identity::request_scope();
        let principal =
            model::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    document_identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_advance(&self.workflow, instance)
            .unwrap()
            .execute()
    }
    pub fn retain_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        key: u64,
    ) -> AssessmentOwner {
        let runtime = self.workflow.runtime();
        let scope = model::operator_identity::request_scope();
        let actor =
            model::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        let request = runtime.request(&actor, &scope).on_branch(instance.branch());
        let owner = request
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    document_identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_advance(&self.workflow, instance)
            .unwrap()
            .into_assessment_demand(model::assessment_output::RetentionAssessmentDemand::new(
                model::retention_entry::DOCUMENT_IDENTITY,
            ))
            .unwrap()
            .controls(WorthQueryOutputDemandControls::new(
                std::num::NonZeroUsize::new(512).unwrap(),
                std::num::NonZeroUsize::new(1024).unwrap(),
            ))
            .retain_operation(self.workflow.program_runtime())
            .unwrap();
        owner
    }
    pub fn assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        key: u64,
    ) -> (
        AssessmentOwner,
        WorthQueryWorkflowAssessmentDemandSettlement<model::schema::DocumentRetentionQuery>,
    ) {
        let mut owner = self.retain_assessment(instance.clone(), key);
        let scope = model::operator_identity::request_scope();
        let actor = model::operator_identity::authenticate_operator(
            self.workflow.runtime().installed_schema(),
            &scope,
        );
        let request = self
            .workflow
            .runtime()
            .request(&actor, &scope)
            .on_branch(instance.branch());
        let settlement = match owner.advance(&request).unwrap() {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
            _ => panic!("actual producer settles"),
        };
        (owner, settlement)
    }
    pub fn accept_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        settlement: &WorthQueryWorkflowAssessmentDemandSettlement<
            model::schema::DocumentRetentionQuery,
        >,
        key: u64,
    ) -> WorkflowProgressOutcome {
        let runtime = self.workflow.runtime();
        let scope = model::operator_identity::request_scope();
        let actor =
            model::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        runtime
            .request(&actor, &scope)
            .on_branch(instance.branch())
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    document_identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_advance(&self.workflow, instance)
            .unwrap()
            .accept_assessment(settlement)
            .unwrap()
    }
    pub fn approve(
        &self,
        instance: PublishedWorkflowInstanceRef,
        approval: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        key: u64,
    ) {
        let runtime = self.workflow.runtime();
        let scope = model::operator_identity::request_scope();
        let actor =
            model::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
        let signing = runtime
            .request(&actor, &scope)
            .on_branch(instance.branch())
            .mutate(WorkflowApprovalIntent {
                input: WorkflowAdvanceInput {
                    document_identity: model::retention_entry::DOCUMENT_IDENTITY.into(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_approval(
                &self.workflow,
                instance,
                approval,
                proposal,
                WorkflowApprovalDecision::Approve,
            )
            .unwrap();
        let intent = signing
            .authentication_intent()
            .cloned()
            .expect("real signature challenge");
        let event = model::operator_identity::block_on(
            self.workflow
                .authentication()
                .authenticate((), &actor, intent, &scope),
        )
        .unwrap();
        assert!(matches!(
            signing.sign(&event).unwrap().execute(),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
}
