//! Original typed command locators. This carries neither an actor nor a grant.
use super::*;
use worth_query_declaration::facade::{
    application_capability::ApplicationWorkflowRecordedActorSource,
    application_operation::ApplicationMutationBinding,
    application_program::ApplicationProgramRevision,
};

/// Opaque original typed command locators for native capability observation.
/// Only the designated publication owner constructs this context. It carries
/// no actor value, live authority slot or grant, and cannot itself admit access.
#[derive(Clone, Debug)]
pub struct WorthQueryWorkflowActorContext {
    pub(crate) runtime: u64,
    pub(crate) instance: super::super::PublishedWorkflowInstanceRef,
    pub(crate) command: Command,
    pub(crate) revision: ApplicationProgramRevision,
    pub(crate) spec: String,
    pub(crate) support: [u8; 32],
    pub(crate) resources:
        worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling,
    pub(crate) source: ApplicationWorkflowRecordedActorSource,
    pub(crate) operation: String,
    pub(crate) binding: String,
    capability: String,
    capability_type: String,
}
#[derive(Clone, Debug)]
pub struct OriginalActorSources {
    pub proposal_input_identity: [u8; 32],
    pub parent_instance: worth_relational::facade::identity::EntityId,
    pub proposal: worth_relational::facade::identity::EntityId,
    pub proposal_identity: String,
    pub transition:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator,
    pub transition_identity: String,
    pub approval_parent: Option<worth_relational::facade::identity::EntityId>,
    pub approval: Option<
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator,
    >,
}

#[derive(Clone, Debug)]
pub struct OperationActorCommand {
    pub captured_input:
        Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    pub instance: worth_relational::facade::identity::EntityId,
    pub branch: crate::basis::WorthQueryProductBranch,
    pub node_path: String,
    pub operation: String,
    pub binding: Option<String>,
    pub input_identity: [u8; 32],
    pub original_sources: Option<OriginalActorSources>,
}

#[derive(Clone, Debug)]
pub enum Command {
    Approval {
        required: RequiredWorkflowApproval,
        proposal: super::super::PublishedWorkflowProposalRef,
    },
    Operation {
        required: OperationActorCommand,
    },
}

impl WorthQueryWorkflowActorContext {
    pub fn build<Schema, Spec, Binding>(
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        runtime: u64,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
            Schema,
            Spec,
        >,
        instance: &super::super::PublishedWorkflowInstanceRef,
        command: Command,
    ) -> Result<
        Option<Self>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
            Schema = Schema,
        >,
        Binding: ApplicationMutationBinding<Schema>,
    {
        use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
        let policy = installed.actor_policy();
        let mut entries = policy.bindings().iter().filter(|entry| {
            entry.operation() == Binding::Operation::IDENTIFIER
                && entry.binding() == Binding::IDENTITY
        });
        let Some(entry) = entries.next() else {
            return Ok(None);
        };
        if entries.next().is_some() {
            return Err(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                    "workflow actor binding is not singular",
                ),
            );
        }
        Ok(Some(Self {
            runtime,
            instance: instance.clone(),
            command,
            revision: *installed.program_revision(),
            spec: Spec::IDENTITY.as_str().to_owned(),
            support: *installed.support_identity_bytes(),
            resources: installed.resources(),
            source: entry.source(),
            operation: Binding::Operation::IDENTIFIER.to_owned(),
            binding: Binding::IDENTITY.to_owned(),
            capability: entry.capability().0.to_owned(),
            capability_type: entry.capability().1.to_owned(),
        }))
    }

    pub fn matches_request(
        &self,
        runtime: u64,
        branch: crate::basis::WorthQueryProductBranch,
        operation: &str,
        capability: &str,
        capability_type: &str,
    ) -> bool {
        self.runtime == runtime
            && self.instance.branch() == branch
            && self.operation == operation
            && self.capability == capability
            && self.capability_type == capability_type
    }
}
