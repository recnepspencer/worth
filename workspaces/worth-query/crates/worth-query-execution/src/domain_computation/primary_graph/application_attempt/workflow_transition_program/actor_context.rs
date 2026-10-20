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
    pub(super) runtime: u64,
    pub(super) instance: super::super::PublishedWorkflowInstanceRef,
    pub(super) command: Command,
    pub(super) revision: ApplicationProgramRevision,
    pub(super) spec: String,
    pub(super) support: [u8; 32],
    pub(super) resources:
        worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling,
    pub(super) source: ApplicationWorkflowRecordedActorSource,
    pub(super) operation: String,
    pub(super) binding: String,
    capability: String,
    capability_type: String,
}
#[derive(Clone, Debug)]
pub(super) struct OriginalActorSources {
    pub(super) parent_instance: worth_relational::facade::identity::EntityId,
    pub(super) proposal: worth_relational::facade::identity::EntityId,
    pub(super) proposal_identity: String,
    pub(super) transition:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator,
    pub(super) transition_identity: String,
    pub(super) approval_parent: Option<worth_relational::facade::identity::EntityId>,
    pub(super) approval: Option<
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator,
    >,
}

#[derive(Clone, Debug)]
pub(super) struct OperationActorCommand {
    pub(super) instance: worth_relational::facade::identity::EntityId,
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) node_path: String,
    pub(super) operation: String,
    pub(super) binding: Option<String>,
    pub(super) input_identity: [u8; 32],
    pub(super) original_sources: Option<OriginalActorSources>,
}

#[derive(Clone, Debug)]
pub(super) enum Command {
    Approval {
        required: RequiredWorkflowApproval,
        proposal: super::super::PublishedWorkflowProposalRef,
    },
    Operation {
        required: OperationActorCommand,
    },
}

impl WorthQueryWorkflowAdvanceAdapter {
    /// Publication alone supplies the original typed command before admission.
    /// The context is an untrusted locator; native selection checks its facts.
    pub fn bind_approval_actor_context<Schema, Spec, Binding>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        workflow: crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: &super::super::PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &super::super::PublishedWorkflowProposalRef,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let command = Command::Approval {
            required: required.clone(),
            proposal: proposal.clone(),
        };
        context::<Schema, Spec, Binding>(
            selected.application().runtime.authority_identity().as_u64(),
            workflow.workflow_spec_for(selected),
            instance,
            command,
        )
    }
    pub fn bind_operation_actor_context<Schema, Spec, Binding>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        context::<Schema, Spec, Binding>(
            required.original_runtime(),
            installed,
            required.original_instance(),
            Command::Operation {
                required: OperationActorCommand {
                    instance: required.instance(),
                    branch: required.branch(),
                    node_path: required.node_path().to_owned(),
                    operation: required.operation().to_owned(),
                    binding: required.binding().map(str::to_owned),
                    input_identity: *required.input_identity(),
                    original_sources: required.original_sources().cloned(),
                },
            },
        )
    }
}
fn context<Schema, Spec, Binding>(
    runtime: u64,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    instance: &super::super::PublishedWorkflowInstanceRef,
    command: Command,
) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Binding: ApplicationMutationBinding<Schema>,
{
    let policy = installed.actor_policy();
    let mut entries = policy.bindings().iter().filter(|entry| {
        entry.operation() == Binding::Operation::IDENTIFIER && entry.binding() == Binding::IDENTITY
    });
    let Some(entry) = entries.next() else {
        return Ok(None);
    };
    if entries.next().is_some() {
        return Err(affinity("workflow actor binding is not singular"));
    }
    Ok(Some(WorthQueryWorkflowActorContext {
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
fn affinity(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

impl WorthQueryWorkflowActorContext {
    pub(in crate::domain_computation) fn matches_request(
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
