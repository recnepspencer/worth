use crate::domain_computation::primary_graph::application_attempt::OutputSourceOps;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;

#[must_use = "prepared workflow instance start owns a live application attempt"]
pub struct PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) definition: worth_relational::facade::identity::EntityId,
    pub(super) definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) discovery_intent: super::WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    pub(super) instance_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) start_path: String,
    pub(super) supersession: Option<super::supersession::WorkflowStartSupersession>,
}



/// A workflow instance start that landed: the instance, its commit receipt, and whether the
/// idempotency key replayed an already-landed commit.
#[derive(Debug)]
pub struct PerformedWorkflowInstanceStart {
    instance: PublishedWorkflowInstanceRef,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    replayed: bool,
}

impl PerformedWorkflowInstanceStart {
    pub const fn instance(&self) -> &PublishedWorkflowInstanceRef {
        &self.instance
    }

    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

/// What executing a workflow instance start produced. `ProjectionDenied` means the commit
/// landed but its receipt did not record exactly one started instance.
#[derive(Debug)]
pub enum WorkflowInstanceStartOutcome {
    Started(PerformedWorkflowInstanceStart),
    /// The branch holds another definition current. Nothing was written.
    Superseded(super::supersession::SupersededWorkflowDefinitionStart),
    /// The definition's lineage is retired. Nothing was written.
    Retired(super::supersession::RetiredWorkflowDefinitionStart),
    /// The commit did not land. A landed commit, first or replayed, is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

pub(in crate::domain_computation::primary_graph) trait WorkflowInstanceProgramPublicationScopedOps<
    Schema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_workflow_instance_start<Operation: 'static, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceStartOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(self.product_runtime()) {
            return WorkflowInstanceStartOutcome::Application(
                crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
                .landed()
                .expect_err("a foreign phase cannot commit"),
            );
        }
        let PreparedWorkflowInstanceStart {
            program,
            program_revision,
            definition,
            definition_content_identity,
            discovery_intent,
            instance_identity_locator,
            start_path,
            supersession,
        } = prepared;
        let presented = match self
            .installed_program_support()
            .ok_or_else(
                super::super::WorthQueryApplicationCommitDenial::application_program_required,
            )
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowInstanceStartOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            execution,
            phase,
            &presented,
            program,
            idempotency.bind_workflow_instance(discovery_intent.intent_identity()),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        match outcome.landed() {
            Ok((receipt, replayed)) => project(
                receipt,
                definition_content_identity,
                definition,
                program_revision,
                discovery_intent.lookup_identity(),
                instance_identity_locator,
                start_path,
                replayed,
            ),
            Err(uncommitted) => match supersession {
                Some(supersession) => supersession
                    .outcome(uncommitted)
                    .unwrap_or_else(WorkflowInstanceStartOutcome::Application),
                None => WorkflowInstanceStartOutcome::Application(uncommitted),
            },
        }
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> WorkflowInstanceProgramPublicationScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

fn project(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
    definition: worth_relational::facade::identity::EntityId,
    program_revision: ApplicationProgramRevision,
    instance_identity: &str,
    instance_identity_locator: worth_foundational::facade::AspectFieldLocator,
    start_path: String,
    replayed: bool,
) -> WorkflowInstanceStartOutcome {
    match project_instance(
        &receipt,
        &definition_content_identity,
        definition,
        program_revision,
        instance_identity,
        &instance_identity_locator,
        &start_path,
    ) {
        Some(instance) => WorkflowInstanceStartOutcome::Started(PerformedWorkflowInstanceStart {
            instance,
            receipt,
            replayed,
        }),
        None => WorkflowInstanceStartOutcome::ProjectionDenied(receipt),
    }
}

pub(super) fn project_instance(
    receipt: &super::super::WorthQueryApplicationCommitReceipt,
    definition_content_identity: &ApplicationWorkflowDefinitionContentIdentity,
    definition: worth_relational::facade::identity::EntityId,
    program_revision: ApplicationProgramRevision,
    instance_identity: &str,
    instance_identity_locator: &worth_foundational::facade::AspectFieldLocator,
    start_path: &str,
) -> Option<PublishedWorkflowInstanceRef> {
    let expected = AspectValue::String(InternedString::Raw(instance_identity.to_owned()));
    let locator = &instance_identity_locator;
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            (receipt
                .committed_changes()
                .committed_field_values(entity, &[locator])
                == Some(vec![expected.clone()]))
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    let [entity_id] = candidates.as_slice() else {
        return None;
    };
    Some(PublishedWorkflowInstanceRef {
        branch: receipt.product_branch(),
        started_on: receipt.product_branch(),
        entity_id: *entity_id,
        definition_content_identity: definition_content_identity.clone(),
        definition,
        program_revision,
        start_path: start_path.to_owned(),
    })
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>
{
    /// Owns original creation intent without retaining this attempt or granting a read.
    pub fn discovery_intent(&self) -> super::WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
        self.discovery_intent.clone()
    }

    /// Controls retained as facts; each host call obtains fresh execution admission.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.request_scope()
    }
}
