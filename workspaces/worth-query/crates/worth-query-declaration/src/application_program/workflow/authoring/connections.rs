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

    /// Binds the permitted current node, return destination and actual business
    /// operation once. Native execution must publish its body and navigation together.
    pub fn back_action<Binding, SourceKind, TargetKind>(
        &mut self,
        source: &impl ApplicationWorkflowOutputBinding<SourceKind>,
        target: &impl ApplicationWorkflowInputBinding<TargetKind>,
    ) -> &mut Self
    where
        Binding: crate::application_operation::ApplicationMutationBinding<Spec::Schema>,
    {
        self.connect(source, target, ApplicationWorkflowConnectionKind::BackAction(
            crate::application_program::workflow::ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>(),
        ))
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

    /// Declare target-specific original input capture from a native control node.
    /// Pure meaning only: installation must support this profile before publication.
    /// Validation requires a bound control source and a bound guarded target.
    ///
    /// ```
    /// use worth_query_declaration::facade::application_program::{
    ///     ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowSpec,
    ///     ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode,
    /// };
    /// fn connect<S: ApplicationWorkflowSpec>(
    ///     graph: &mut ApplicationWorkflowDefinitionBuilder<S>,
    ///     proposal: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    ///     target: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    /// ) { graph.captured_operation_input(proposal, target); }
    /// ```
    pub fn captured_operation_input<Source, Target>(
        &mut self,
        source: &Source,
        target: &Target,
    ) -> &mut Self
    where
        Source: ApplicationWorkflowOutputBinding<ApplicationWorkflowOperationNode>,
        Target: ApplicationWorkflowInputBinding<ApplicationWorkflowOperationNode>,
    {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::CapturedOperationInput,
        )
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
