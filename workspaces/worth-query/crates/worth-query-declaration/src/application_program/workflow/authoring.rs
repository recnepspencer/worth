use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_operation::ApplicationMutationBinding,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::{
        ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
        ApplicationOperationMarkerIdentity,
    },
};

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowAwaitInbound, ApplicationWorkflowCondition,
    ApplicationWorkflowConditionOperands, ApplicationWorkflowConnection,
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowInboundRef,
    ApplicationWorkflowInboundWait, ApplicationWorkflowNode, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowNodeKind, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    ApplicationWorkflowSubjectSelector, AuthoredWorkflowDefinition,
};

mod command;
mod component;
mod connections;
mod denial;
pub use command::{ApplicationWorkflowAuthoringCommand, ApplicationWorkflowCommandAdapter};
pub use component::{
    ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentInputBinding,
    ApplicationWorkflowComponentInputPort, ApplicationWorkflowComponentNodeRef,
    ApplicationWorkflowComponentOutputBinding, ApplicationWorkflowComponentOutputPort,
    AuthoredWorkflowComponent, ExpandedWorkflowComponent, ExpandedWorkflowComponentInComponent,
};
pub use denial::{ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentResource};

pub enum ApplicationWorkflowOperationNode {}
pub enum ApplicationWorkflowAwaitInboundNode {}
pub enum ApplicationWorkflowAssessmentNode {}
pub enum ApplicationWorkflowConditionNode {}
pub enum ApplicationWorkflowApprovalNode {}
pub enum ApplicationWorkflowEvidenceJoinNode {}
pub enum ApplicationWorkflowTerminalNode {}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowNodeRef<Kind> {
    identity: ApplicationWorkflowNodeIdentity,
    marker: PhantomData<fn() -> Kind>,
}

impl<Kind> Clone for ApplicationWorkflowNodeRef<Kind> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            marker: PhantomData,
        }
    }
}

impl<Kind> ApplicationWorkflowNodeRef<Kind> {
    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }

    pub(super) fn qualified(
        &self,
        occurrence: &str,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self {
            identity: self
                .identity
                .prefixed(occurrence)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
            marker: PhantomData,
        })
    }
}

mod binding_seal {
    pub trait Sealed {}
}

#[allow(private_bounds)]
pub trait ApplicationWorkflowInputBinding<Kind>: binding_seal::Sealed {
    #[doc(hidden)]
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity;
}

#[allow(private_bounds)]
pub trait ApplicationWorkflowOutputBinding<Kind>: binding_seal::Sealed {
    #[doc(hidden)]
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity;
}

impl<Kind> binding_seal::Sealed for ApplicationWorkflowNodeRef<Kind> {}

impl<Kind> ApplicationWorkflowInputBinding<Kind> for ApplicationWorkflowNodeRef<Kind> {
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }
}

impl<Kind> ApplicationWorkflowOutputBinding<Kind> for ApplicationWorkflowNodeRef<Kind> {
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }
}

pub struct ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    identity: ApplicationWorkflowDefinitionIdentity,
    limits: ApplicationWorkflowDefinitionLimits,
    start: Option<ApplicationWorkflowNodeIdentity>,
    nodes: Vec<ApplicationWorkflowNode>,
    node_is_operation: BTreeMap<ApplicationWorkflowNodeIdentity, bool>,
    connections: Vec<ApplicationWorkflowConnection>,
    component_expansions: Vec<super::ApplicationWorkflowComponentExpansion>,
    component_occurrences: BTreeSet<String>,
    component_expansion_usage: component::ComponentExpansionUsage,
    marker: PhantomData<fn() -> Spec>,
}

impl<Spec> ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn new(
        identity: impl Into<String>,
        limits: ApplicationWorkflowDefinitionLimits,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self {
            identity: ApplicationWorkflowDefinitionIdentity::new(identity)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
            limits,
            start: None,
            nodes: Vec::new(),
            node_is_operation: BTreeMap::new(),
            connections: Vec::new(),
            component_expansions: Vec::new(),
            component_occurrences: BTreeSet::new(),
            component_expansion_usage: component::ComponentExpansionUsage::default(),
            marker: PhantomData,
        })
    }

    pub fn operation<Operation>(
        &mut self,
        identity: impl Into<String>,
        requires_workflow_authority: bool,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Operation {
                operation: ApplicationWorkflowOperationRef::declared::<Spec, Operation>(),
                requires_workflow_authority,
            },
        )
    }

    /// Wait for the exact installed inbound completion of a prior operation.
    /// This declaration does not receive an occurrence or resume an instance.
    pub fn await_inbound<Effect>(
        &mut self,
        identity: impl Into<String>,
        origin: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        inbound: ApplicationInboundOccurrenceBinding<Effect>,
        wait: ApplicationWorkflowInboundWait,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowAwaitInboundNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Effect: ApplicationEffectMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::AwaitInbound(ApplicationWorkflowAwaitInbound::new(
                origin.identity().clone(),
                ApplicationWorkflowInboundRef::declared::<Spec, Effect>(inbound),
                wait,
            )),
        )
    }

    pub fn assessment<Query>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.assessment_for::<Query>(identity, ApplicationWorkflowSubjectSelector::Resource)
    }

    pub fn assessment_for<Query>(
        &mut self,
        identity: impl Into<String>,
        subject: ApplicationWorkflowSubjectSelector,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Assessment(
                ApplicationWorkflowAssessmentRef::declared_for::<Spec, Query>(subject),
            ),
        )
    }

    pub fn assessment_when_related_relation_present<Query, Relation, From, To>(
        &mut self,
        identity: impl Into<String>,
        relation: crate::application_schema::ApplicationRelationRef<
            Spec::Schema,
            Relation,
            From,
            To,
        >,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Assessment(
                ApplicationWorkflowAssessmentRef::declared_when_related_relation_present::<
                    Spec,
                    Query,
                    Relation,
                    From,
                    To,
                >(relation),
            ),
        )
    }

    pub fn operation_binding<Binding>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Binding: ApplicationMutationBinding<Spec::Schema>,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Operation {
                operation: ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>(),
                requires_workflow_authority: Binding::REQUIRES_WORKFLOW_AUTHORITY,
            },
        )
    }

    /// A condition node deciding by the Bool expression `source` over the
    /// query results `operands` name.
    pub fn condition(
        &mut self,
        identity: impl Into<String>,
        source: &str,
        operands: ApplicationWorkflowConditionOperands<Spec>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowConditionNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        let condition = ApplicationWorkflowCondition::parse(source, operands.into_operands())
            .map_err(ApplicationWorkflowAuthoringDenial::Condition)?;
        self.push_node(identity, ApplicationWorkflowNodeKind::Condition(condition))
    }

    pub fn approval<Capability>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Approval(ApplicationWorkflowApprovalRef::declared::<
                Spec,
                Capability,
            >()),
        )
    }

    pub fn evidence_join(
        &mut self,
        identity: impl Into<String>,
        policy: ApplicationWorkflowEvidenceJoinPolicy,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.push_node(identity, ApplicationWorkflowNodeKind::EvidenceJoin(policy))
    }

    pub fn terminal(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowTerminalNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.push_node(identity, ApplicationWorkflowNodeKind::Terminal)
    }

    pub fn expand_component(
        &mut self,
        occurrence: &str,
        component: &AuthoredWorkflowComponent<Spec>,
    ) -> Result<ExpandedWorkflowComponent, ApplicationWorkflowAuthoringDenial> {
        component::expand(self, occurrence, component)
    }

    pub fn finish(
        self,
    ) -> Result<AuthoredWorkflowDefinition<Spec>, ApplicationWorkflowAuthoringDenial> {
        Ok(AuthoredWorkflowDefinition {
            identity: self.identity,
            limits: self.limits,
            start: self.start,
            nodes: self.nodes,
            connections: self.connections,
            component_expansions: self.component_expansions,
            marker: PhantomData,
        })
    }

    fn push_node<Kind>(
        &mut self,
        identity: impl Into<String>,
        kind: ApplicationWorkflowNodeKind,
    ) -> Result<ApplicationWorkflowNodeRef<Kind>, ApplicationWorkflowAuthoringDenial> {
        let identity = ApplicationWorkflowNodeIdentity::new(identity)
            .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
        self.push_erased(identity.clone(), kind)?;
        Ok(ApplicationWorkflowNodeRef {
            identity,
            marker: PhantomData,
        })
    }

    fn push_erased(
        &mut self,
        identity: ApplicationWorkflowNodeIdentity,
        kind: ApplicationWorkflowNodeKind,
    ) -> Result<(), ApplicationWorkflowAuthoringDenial> {
        if self.node_is_operation.contains_key(&identity) {
            return Err(ApplicationWorkflowAuthoringDenial::DuplicateNode(identity));
        }
        self.node_is_operation
            .insert(identity.clone(), kind.is_operation());
        self.nodes
            .push(ApplicationWorkflowNode::new(identity, kind));
        Ok(())
    }
}
