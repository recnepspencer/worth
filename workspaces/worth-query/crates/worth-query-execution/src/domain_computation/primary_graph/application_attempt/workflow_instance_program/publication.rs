use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[must_use = "prepared workflow instance start owns a live application attempt"]
pub struct PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) definition: worth_relational::facade::identity::EntityId,
    pub(super) definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) instance_identity: String,
    pub(super) instance_intent_identity: [u8; 32],
    pub(super) instance_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) start_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowInstanceRef {
    branch: crate::basis::WorthQueryProductBranch,
    entity_id: worth_relational::facade::identity::EntityId,
    definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
    definition: worth_relational::facade::identity::EntityId,
    program_revision: ApplicationProgramRevision,
    start_path: String,
}

impl PublishedWorkflowInstanceRef {
    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub const fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.entity_id
    }

    pub const fn definition_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.definition
    }

    /// The revision the instance started under. Program adoption may carry
    /// the instance to a later revision; advancing reads the revision from
    /// instance truth and never from this reference.
    pub const fn program_revision(&self) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub const fn definition_content_identity(
        &self,
    ) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.definition_content_identity
    }

    pub fn start_node_path(&self) -> &str {
        &self.start_path
    }
}

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

#[derive(Debug)]
pub enum WorkflowInstanceStartOutcome {
    Started(PerformedWorkflowInstanceStart),
    Application(WorthQueryApplicationCommitOutcome),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_instance_start<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceStartOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let PreparedWorkflowInstanceStart {
            program,
            program_revision,
            definition,
            definition_content_identity,
            instance_identity,
            instance_intent_identity,
            instance_identity_locator,
            start_path,
        } = prepared;
        let Some(presented) = self
            .installed_program_support()
            .and_then(|support| support.present(&program_revision))
        else {
            return WorkflowInstanceStartOutcome::Application(
                WorthQueryApplicationCommitOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_instance(&instance_intent_identity),
        );
        match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => project(
                receipt,
                definition_content_identity,
                definition,
                program_revision.clone(),
                instance_identity,
                instance_identity_locator,
                start_path,
                false,
            ),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => project(
                receipt,
                definition_content_identity,
                definition,
                program_revision,
                instance_identity,
                instance_identity_locator,
                start_path,
                true,
            ),
            other => WorkflowInstanceStartOutcome::Application(other),
        }
    }
}

fn project(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
    definition: worth_relational::facade::identity::EntityId,
    program_revision: ApplicationProgramRevision,
    instance_identity: String,
    instance_identity_locator: worth_foundational::facade::AspectFieldLocator,
    start_path: String,
    replayed: bool,
) -> WorkflowInstanceStartOutcome {
    let expected = AspectValue::String(InternedString::Raw(instance_identity));
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
        return WorkflowInstanceStartOutcome::ProjectionDenied(receipt);
    };
    WorkflowInstanceStartOutcome::Started(PerformedWorkflowInstanceStart {
        instance: PublishedWorkflowInstanceRef {
            branch: receipt.product_branch(),
            entity_id: *entity_id,
            definition_content_identity,
            definition,
            program_revision,
            start_path,
        },
        receipt,
        replayed,
    })
}
