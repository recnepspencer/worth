use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationIdempotencyResolutionDenial,
};
use crate::domain_computation::primary_graph::workflow::proposal::derive_workflow_proposal_context_identity;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[must_use = "prepared workflow proposal owns a live application attempt"]
pub struct PreparedWorkflowProposal<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) transition_identity_locator: AspectFieldLocator,
    pub(super) proposal_identity_locator: AspectFieldLocator,
    pub(super) proposal_node_path_locator: AspectFieldLocator,
    pub(super) proposal_context_identity: [u8; 32],
    pub(super) instance: PublishedWorkflowInstanceRef,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) input_identity: [u8; 32],
    pub(super) source_identity: Option<[u8; 32]>,
    pub(super) progress_update: Option<
        crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate,
    >,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowProposalRef {
    branch: crate::basis::WorthQueryProductBranch,
    entity: EntityId,
    instance: EntityId,
    definition: EntityId,
    identity: String,
    operation: String,
    input_type: String,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    node_path: String,
}

impl PublishedWorkflowProposalRef {
    pub const fn entity_id(&self) -> EntityId {
        self.entity
    }
    pub const fn instance_entity_id(&self) -> EntityId {
        self.instance
    }
    pub const fn definition_entity_id(&self) -> EntityId {
        self.definition
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn input_type(&self) -> &str {
        &self.input_type
    }
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }
    pub const fn source_identity(&self) -> Option<&[u8; 32]> {
        self.source_identity.as_ref()
    }
    pub fn node_path(&self) -> &str {
        &self.node_path
    }
    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }
}

#[derive(Debug)]
pub struct PerformedWorkflowProposal {
    proposal: PublishedWorkflowProposalRef,
    transition: EntityId,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    replayed: bool,
}

impl PerformedWorkflowProposal {
    pub const fn proposal(&self) -> &PublishedWorkflowProposalRef {
        &self.proposal
    }
    pub const fn transition(&self) -> EntityId {
        self.transition
    }
    pub fn node_path(&self) -> &str {
        self.proposal.node_path()
    }
    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

#[derive(Debug)]
pub enum WorkflowProposalOutcome {
    Published(PerformedWorkflowProposal),
    Application(WorthQueryApplicationCommitOutcome),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn resolve_workflow_proposal_replay<
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        instance: PublishedWorkflowInstanceRef,
        operation: String,
        input_type: String,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
    ) -> Result<Option<WorkflowProposalOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        let context_identity = derive_workflow_proposal_context_identity(instance.entity_id());
        let resolution = self
            .resolve_admitted_application_idempotency(
                admission,
                idempotency.bind_workflow_proposal_context(&context_identity),
            )?
            .into_resolution();
        let outcome = match resolution {
            WorthQueryApplicationIdempotencyResolution::Unseen => return Ok(None),
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                WorkflowProposalOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                ))
            }
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                let layout = self
                    .runtime
                    .primary_graph()
                    .expect("application runtime requires primary graph")
                    .layout()
                    .workflow();
                project(
                    receipt,
                    ProjectionContext {
                        transition_identity_locator: layout.transition.identity.clone(),
                        proposal_identity_locator: layout.proposal.identity.clone(),
                        proposal_node_path_locator: layout.proposal.node_path.clone(),
                        instance,
                        operation,
                        input_type,
                        input_identity,
                        source_identity,
                    },
                    true,
                )
            }
        };
        Ok(Some(outcome))
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_proposal<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProposalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let PreparedWorkflowProposal {
            program,
            program_revision,
            transition_identity_locator,
            proposal_identity_locator,
            proposal_node_path_locator,
            proposal_context_identity,
            instance,
            operation,
            input_type,
            input_identity,
            source_identity,
            progress_update,
        } = prepared;
        let Some(presented) = self
            .installed_program_support()
            .and_then(|support| support.present(&program_revision))
        else {
            return WorkflowProposalOutcome::Application(
                WorthQueryApplicationCommitOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_proposal_context(&proposal_context_identity),
        );
        let context = ProjectionContext {
            transition_identity_locator,
            proposal_identity_locator,
            proposal_node_path_locator,
            instance,
            operation,
            input_type,
            input_identity,
            source_identity,
        };
        let projected = match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                project(receipt, context, false)
            }
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
                project(receipt, context, true)
            }
            other => WorkflowProposalOutcome::Application(other),
        };
        if let (Some(progress_update), WorkflowProposalOutcome::Published(performed)) =
            (progress_update, &projected)
        {
            let committed_revision = performed.receipt().commit_reference().version_id;
            let progress_key = progress_update.key();
            let _ = self
                .primary_provider
                .graph
                .with_workflow_instance_progress_mut(progress_key, |retention| {
                    progress_update.apply(retention, committed_revision)
                });
        }
        projected
    }
}

struct ProjectionContext {
    transition_identity_locator: AspectFieldLocator,
    proposal_identity_locator: AspectFieldLocator,
    proposal_node_path_locator: AspectFieldLocator,
    instance: PublishedWorkflowInstanceRef,
    operation: String,
    input_type: String,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
}

fn project(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    context: ProjectionContext,
    replayed: bool,
) -> WorkflowProposalOutcome {
    if receipt.product_branch() != context.instance.branch() {
        return WorkflowProposalOutcome::ProjectionDenied(receipt);
    }
    let transition = unique_created_text(&receipt, &context.transition_identity_locator);
    let proposal = unique_created_text_pair(
        &receipt,
        &context.proposal_identity_locator,
        &context.proposal_node_path_locator,
    );
    let (Some((transition, _)), Some((proposal, identity, node_path))) = (transition, proposal)
    else {
        return WorkflowProposalOutcome::ProjectionDenied(receipt);
    };
    let published = PublishedWorkflowProposalRef {
        branch: receipt.product_branch(),
        entity: proposal,
        instance: context.instance.entity_id(),
        definition: context.instance.definition_entity_id(),
        identity,
        operation: context.operation,
        input_type: context.input_type,
        input_identity: context.input_identity,
        source_identity: context.source_identity,
        node_path,
    };
    WorkflowProposalOutcome::Published(PerformedWorkflowProposal {
        proposal: published,
        transition,
        receipt,
        replayed,
    })
}

fn unique_created_text(
    receipt: &super::super::WorthQueryApplicationCommitReceipt,
    locator: &AspectFieldLocator,
) -> Option<(EntityId, String)> {
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            receipt
                .committed_changes()
                .committed_field_values(entity, &[locator])
                .and_then(|values| match values.as_slice() {
                    [AspectValue::String(InternedString::Raw(value))] => {
                        Some((entity, value.clone()))
                    }
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate.clone())
}

fn unique_created_text_pair(
    receipt: &super::super::WorthQueryApplicationCommitReceipt,
    first: &AspectFieldLocator,
    second: &AspectFieldLocator,
) -> Option<(EntityId, String, String)> {
    let candidates = receipt.committed_changes().entity_changes()
        .filter(|(_, change)| *change == worth_relational::facade::publication::RecordStructuralChange::Created)
        .filter_map(|(entity, _)| receipt.committed_changes().committed_field_values(entity, &[first, second])
            .and_then(|values| match values.as_slice() {
                [AspectValue::String(InternedString::Raw(a)), AspectValue::String(InternedString::Raw(b))] => Some((entity, a.clone(), b.clone())),
                _ => None,
            }))
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate.clone())
}
