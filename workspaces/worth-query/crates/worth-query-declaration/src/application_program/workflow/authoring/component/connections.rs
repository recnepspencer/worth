use super::{ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentNodeRef};
use crate::application_program::workflow::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowRetry, ApplicationWorkflowSpec,
};

use super::super::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentResource,
    ApplicationWorkflowConditionNode, ApplicationWorkflowEvidenceJoinNode,
    ApplicationWorkflowOperationNode,
};

impl<Spec> ApplicationWorkflowComponentBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn control<Source, Target>(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<Source>,
        outcome: ApplicationWorkflowControlOutcome,
        target: &ApplicationWorkflowComponentNodeRef<Target>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.control(&source.inner, outcome, &target.inner);
        Ok(self)
    }

    pub fn retry<Source, Target>(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<Source>,
        retry: ApplicationWorkflowRetry,
        target: &ApplicationWorkflowComponentNodeRef<Target>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.retry(&source.inner, retry, &target.inner);
        Ok(self)
    }

    pub fn proposal_for_assessment(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowAssessmentNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph
            .proposal_for_assessment(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn condition_subject(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowConditionNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.condition_subject(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn proposal_for_approval(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowApprovalNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph
            .proposal_for_approval(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn assessment_evidence(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowAssessmentNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowEvidenceJoinNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.assessment_evidence(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn joined_evidence(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowApprovalNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.joined_evidence(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn approval_authority(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowApprovalNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.approval_authority(&source.inner, &target.inner);
        Ok(self)
    }

    pub fn operation_input(
        &mut self,
        source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        self.require_connection(source, target)?;
        self.graph.operation_input(&source.inner, &target.inner);
        Ok(self)
    }

    fn require_connection<Source, Target>(
        &self,
        source: &ApplicationWorkflowComponentNodeRef<Source>,
        target: &ApplicationWorkflowComponentNodeRef<Target>,
    ) -> Result<(), ApplicationWorkflowAuthoringDenial> {
        self.require_owned(source)?;
        self.require_owned(target)?;
        if self.graph.connections.len() >= usize::from(self.graph.limits.maximum_connections()) {
            Err(
                ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                    resource: ApplicationWorkflowComponentResource::Connections,
                    maximum: u32::from(self.graph.limits.maximum_connections()),
                },
            )
        } else {
            Ok(())
        }
    }
}
