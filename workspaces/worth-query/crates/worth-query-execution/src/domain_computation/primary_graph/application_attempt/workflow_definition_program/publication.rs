use crate::domain_computation::primary_graph::application_attempt::EntryCustodyScopedOps;
use crate::domain_computation::primary_graph::application_attempt::OutputSourceOps;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef;
use {
    super::super::WorthQueryApplicationEffectProgram,
    super::super::WorthQueryApplicationUncommitted,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
};

/// Opaque, live application-attempt publication program.
///
/// Construction consumes an admitted projected read set and an owner-bound
/// definition contract. Callers cannot provide lineage IDs, current-pointer
/// relation IDs, effects, or branch affinity.
///
/// ```compile_fail,E0451
/// use worth_query_execution::publication_boundary::workflow_definition_publication::PreparedWorkflowDefinitionPublication;
///
/// fn cannot_forge<Schema, Operation, Input, Scope>()
///     -> PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>
/// {
///     PreparedWorkflowDefinitionPublication {
///         program: unsafe { std::mem::zeroed() },
///         program_revision: unsafe { std::mem::zeroed() },
///         content_identity: unsafe { std::mem::zeroed() },
///         content_identity_locator: unsafe { std::mem::zeroed() },
///         workflow_intent_identity: [0; 32],
///     }
/// }
/// ```
#[must_use = "prepared workflow definition publication owns a live application attempt"]
pub struct PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) content_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) workflow_intent_identity: [u8; 32],
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>
{
    pub fn content_identity(&self) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }

    fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        ApplicationProgramRevision,
        ApplicationWorkflowDefinitionContentIdentity,
        worth_foundational::facade::AspectFieldLocator,
        [u8; 32],
    ) {
        (
            self.program,
            self.program_revision,
            self.content_identity,
            self.content_identity_locator,
            self.workflow_intent_identity,
        )
    }
}

/// The exact branch-current state an author observed before requesting a
/// definition publication. A revision never overwrites a newer predecessor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowDefinitionExpectedPredecessor {
    Absent,
    Published(PublishedWorkflowDefinitionRef),
}

impl WorkflowDefinitionExpectedPredecessor {
    pub(super) fn belongs_to_branch(&self, branch: crate::basis::WorthQueryProductBranch) -> bool {
        match self {
            Self::Absent => true,
            Self::Published(predecessor) => predecessor.branch == branch,
        }
    }

    pub(super) fn definition_entity(&self) -> Option<worth_relational::facade::identity::EntityId> {
        match self {
            Self::Absent => None,
            Self::Published(predecessor) => Some(predecessor.entity_id),
        }
    }
}

/// A workflow definition publication that landed: the published definition, its commit
/// receipt, and whether the idempotency key replayed an already-landed commit.
#[derive(Debug)]
pub struct PerformedWorkflowDefinitionPublication {
    definition: PublishedWorkflowDefinitionRef,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    replayed: bool,
}

impl PerformedWorkflowDefinitionPublication {
    pub const fn definition(&self) -> &PublishedWorkflowDefinitionRef {
        &self.definition
    }

    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

/// What executing a workflow definition publication produced. `ProjectionDenied` means the
/// commit landed but its receipt did not record exactly one created definition with the
/// published content identity, so no reference was projected.
#[derive(Debug)]
pub enum WorkflowDefinitionPublicationOutcome {
    Published(PerformedWorkflowDefinitionPublication),
    /// The commit did not land. A landed commit, first or replayed, is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

pub(in crate::domain_computation::primary_graph) trait PublicationScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_workflow_definition_publication<Operation: 'static, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionPublicationOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let (
            program,
            program_revision,
            content_identity,
            content_identity_locator,
            workflow_intent_identity,
        ) = prepared.into_parts();
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => {
                return WorkflowDefinitionPublicationOutcome::Application(
                    outcome.landed().expect_err("a refused entry cannot commit"),
                )
            }
        };
        let presented = match self
            .installed_program_support()
            .ok_or_else(
                super::super::WorthQueryApplicationCommitDenial::application_program_required,
            )
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowDefinitionPublicationOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let outcome = entry.compare_and_commit_application_for_program_action(
            execution,
            &presented,
            idempotency.bind_workflow_definition(&workflow_intent_identity),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        match outcome.landed() {
            Ok((receipt, replayed)) => project_published(
                receipt,
                content_identity,
                &content_identity_locator,
                replayed,
            ),
            Err(uncommitted) => WorkflowDefinitionPublicationOutcome::Application(uncommitted),
        }
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> PublicationScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

fn project_published(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    content_identity: ApplicationWorkflowDefinitionContentIdentity,
    locator: &worth_foundational::facade::AspectFieldLocator,
    replayed: bool,
) -> WorkflowDefinitionPublicationOutcome {
    let expected = AspectValue::String(InternedString::Raw(content_identity.to_string()));
    let candidates = receipt
        .committed_changes()
        .created_entities_with_field_value(locator, &expected);
    let [entity_id] = candidates.as_slice() else {
        return WorkflowDefinitionPublicationOutcome::ProjectionDenied(receipt);
    };
    let definition = PublishedWorkflowDefinitionRef {
        branch: receipt.product_branch(),
        entity_id: *entity_id,
        content_identity,
    };
    WorkflowDefinitionPublicationOutcome::Published(PerformedWorkflowDefinitionPublication {
        definition,
        receipt,
        replayed,
    })
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>
{
    /// Controls retained as facts; each host call obtains fresh execution admission.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.request_scope()
    }
}
