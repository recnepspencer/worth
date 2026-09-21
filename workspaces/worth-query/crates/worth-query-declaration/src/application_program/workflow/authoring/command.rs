use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};

use super::{ApplicationWorkflowAuthoringDenial, ApplicationWorkflowDefinitionBuilder};
use crate::application_program::workflow::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowNodeKind, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    AuthoredWorkflowDefinition,
};

pub enum ApplicationWorkflowAuthoringCommand {
    Node(ApplicationWorkflowNodeIdentity, ApplicationWorkflowNodeKind),
    Start(ApplicationWorkflowNodeIdentity),
    Connection(ApplicationWorkflowConnection),
}

impl ApplicationWorkflowAuthoringCommand {
    pub fn operation<Spec, Operation>(
        identity: impl Into<String>,
        requires_workflow_authority: bool,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial>
    where
        Spec: ApplicationWorkflowSpec,
        Operation: ApplicationOperationMarkerIdentity<Spec::Schema> + 'static,
    {
        Self::node(
            identity,
            ApplicationWorkflowNodeKind::Operation {
                operation: ApplicationWorkflowOperationRef::declared::<Spec, Operation>(),
                requires_workflow_authority,
            },
        )
    }

    pub fn assessment<Spec, Query>(
        identity: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial>
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        Self::node(
            identity,
            ApplicationWorkflowNodeKind::Assessment(ApplicationWorkflowAssessmentRef::declared::<
                Spec,
                Query,
            >()),
        )
    }

    pub fn approval<Spec, Capability>(
        identity: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial>
    where
        Spec: ApplicationWorkflowSpec,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema> + 'static,
    {
        Self::node(
            identity,
            ApplicationWorkflowNodeKind::Approval(ApplicationWorkflowApprovalRef::declared::<
                Spec,
                Capability,
            >()),
        )
    }

    pub fn evidence_join(
        identity: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Self::node(identity, ApplicationWorkflowNodeKind::EvidenceJoin)
    }

    pub fn terminal(
        identity: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Self::node(identity, ApplicationWorkflowNodeKind::Terminal)
    }

    pub fn start(identity: impl Into<String>) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self::Start(node_identity(identity)?))
    }

    pub fn control(
        source: impl Into<String>,
        outcome: ApplicationWorkflowControlOutcome,
        target: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Self::connection(
            source,
            target,
            ApplicationWorkflowConnectionKind::Control(outcome),
        )
    }

    pub fn data(
        source: impl Into<String>,
        flow: ApplicationWorkflowDataFlow,
        target: impl Into<String>,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Self::connection(
            source,
            target,
            ApplicationWorkflowConnectionKind::Data(flow),
        )
    }

    fn node(
        identity: impl Into<String>,
        kind: ApplicationWorkflowNodeKind,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self::Node(node_identity(identity)?, kind))
    }

    fn connection(
        source: impl Into<String>,
        target: impl Into<String>,
        kind: ApplicationWorkflowConnectionKind,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self::Connection(ApplicationWorkflowConnection::new(
            node_identity(source)?,
            node_identity(target)?,
            kind,
        )))
    }
}

pub struct ApplicationWorkflowCommandAdapter;

impl ApplicationWorkflowCommandAdapter {
    pub fn author<Spec>(
        identity: impl Into<String>,
        limits: ApplicationWorkflowDefinitionLimits,
        commands: impl IntoIterator<Item = ApplicationWorkflowAuthoringCommand>,
    ) -> Result<AuthoredWorkflowDefinition<Spec>, ApplicationWorkflowAuthoringDenial>
    where
        Spec: ApplicationWorkflowSpec,
    {
        let mut builder = ApplicationWorkflowDefinitionBuilder::<Spec>::new(identity, limits)?;
        for command in commands {
            match command {
                ApplicationWorkflowAuthoringCommand::Node(identity, kind) => {
                    builder.push_erased(identity, kind)?;
                }
                ApplicationWorkflowAuthoringCommand::Start(identity) => {
                    builder.start = Some(identity);
                }
                ApplicationWorkflowAuthoringCommand::Connection(connection) => {
                    builder.connections.push(connection);
                }
            }
        }
        builder.finish()
    }
}

fn node_identity(
    identity: impl Into<String>,
) -> Result<ApplicationWorkflowNodeIdentity, ApplicationWorkflowAuthoringDenial> {
    ApplicationWorkflowNodeIdentity::new(identity)
        .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)
}
