use super::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowConditionNode, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowEvidenceJoinNode, ApplicationWorkflowInputBinding,
    ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode, ApplicationWorkflowOutputBinding,
};
use crate::application_program::workflow::{
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowConnection,
    ApplicationWorkflowConnectionKind, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDataFlow, ApplicationWorkflowRetry, ApplicationWorkflowSpec,
};

impl<Spec> ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    /// Connects operation completion in authored order. Data and authority
    /// bindings remain explicit; non-completion outcomes never acquire a route.
    pub fn sequence(
        &mut self,
        operations: &[ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>],
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        if operations.len() < 2 {
            return Err(ApplicationWorkflowAuthoringDenial::InvalidSequenceLength);
        }
        let mut seen = std::collections::BTreeSet::new();
        for operation in operations {
            match self.node_is_operation.get(operation.identity()) {
                None => {
                    return Err(ApplicationWorkflowAuthoringDenial::UnknownSequenceNode(
                        operation.identity().clone(),
                    ))
                }
                Some(false) => {
                    return Err(
                        ApplicationWorkflowAuthoringDenial::NonOperationSequenceNode(
                            operation.identity().clone(),
                        ),
                    )
                }
                Some(true) => {}
            }
            if !seen.insert(operation.identity()) {
                return Err(ApplicationWorkflowAuthoringDenial::DuplicateSequenceNode(
                    operation.identity().clone(),
                ));
            }
        }
        for pair in operations.windows(2) {
            self.control(
                &pair[0],
                ApplicationWorkflowControlOutcome::Completed,
                &pair[1],
            );
        }
        Ok(self)
    }

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
