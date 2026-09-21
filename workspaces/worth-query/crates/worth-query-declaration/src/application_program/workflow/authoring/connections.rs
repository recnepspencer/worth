use super::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowConditionNode, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowEvidenceJoinNode, ApplicationWorkflowInputBinding,
    ApplicationWorkflowOperationNode, ApplicationWorkflowOutputBinding,
};
use crate::application_program::workflow::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow, ApplicationWorkflowRetry,
    ApplicationWorkflowSpec,
};

impl<Spec> ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn start<Kind, Node>(&mut self, node: &Node) -> &mut Self
    where
        Node: ApplicationWorkflowInputBinding<Kind>,
    {
        self.start = Some(node.workflow_node_identity().clone());
        self
    }

    pub fn control<SourceKind, TargetKind, Source, Target>(
        &mut self,
        source: &Source,
        outcome: ApplicationWorkflowControlOutcome,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<SourceKind>,
        Target: ApplicationWorkflowInputBinding<TargetKind>,
    {
        self.connect(
            source,
            target,
            ApplicationWorkflowConnectionKind::Control(outcome),
        )
    }

    pub fn retry<SourceKind, TargetKind, Source, Target>(
        &mut self,
        source: &Source,
        retry: ApplicationWorkflowRetry,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<SourceKind>,
        Target: ApplicationWorkflowInputBinding<TargetKind>,
    {
        self.connect(
            source,
            target,
            ApplicationWorkflowConnectionKind::Retry(retry),
        )
    }

    pub fn proposal_for_assessment<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowOperationNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowAssessmentNode>,
    {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::AssessmentSubject,
        )
    }

    pub fn condition_subject<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowOperationNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowConditionNode>,
    {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::ConditionSubject,
        )
    }

    pub fn proposal_for_approval<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowOperationNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowApprovalNode>,
    {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::ProposalSubject)
    }

    pub fn assessment_evidence<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowAssessmentNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowEvidenceJoinNode>,
    {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::AssessmentEvidence,
        )
    }

    pub fn joined_evidence<Source, Target>(&mut self, source: &Source, target: &Target) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowEvidenceJoinNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowApprovalNode>,
    {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::JoinedEvidence)
    }

    pub fn approval_authority<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowApprovalNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowOperationNode>,
    {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::ApprovalAuthority,
        )
    }

    pub fn operation_input<Source, Target>(&mut self, source: &Source, target: &Target) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowOperationNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowOperationNode>,
    {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::OperationInput)
    }

    fn connect<SourceKind, TargetKind, Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
        kind: ApplicationWorkflowConnectionKind,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<SourceKind>,
        Target: ApplicationWorkflowInputBinding<TargetKind>,
    {
        self.connections.push(ApplicationWorkflowConnection::new(
            source.workflow_node_identity().clone(),
            target.workflow_node_identity().clone(),
            kind,
        ));
        self
    }

    fn connect_data<SourceKind, TargetKind, Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
        flow: ApplicationWorkflowDataFlow,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<SourceKind>,
        Target: ApplicationWorkflowInputBinding<TargetKind>,
    {
        self.connect(
            source,
            target,
            ApplicationWorkflowConnectionKind::Data(flow),
        )
    }
}
