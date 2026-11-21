use crate::domain_computation::primary_graph::PrimaryGraphHandleWorkflowExt;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationUncommitted,
};
use crate::domain_computation::primary_graph::workflow::proposal::derive_workflow_proposal_context_identity;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
#[must_use = "prepared workflow proposal owns a live application attempt"]
pub struct PreparedWorkflowProposal<Schema, Operation, Input, Scope> {
    pub program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub program_revision: ApplicationProgramRevision,
    pub transition_identity_locator: AspectFieldLocator,
    pub proposal_identity_locator: AspectFieldLocator,
    pub proposal_node_path_locator: AspectFieldLocator,
    pub proposal_context_identity: [u8; 32],
    pub instance: PublishedWorkflowInstanceRef,
    pub operation: String,
    pub input_type: String,
    pub input_identity: [u8; 32],
    pub source_identity: Option<[u8; 32]>,
    pub captured_identity: Option<[u8; 32]>,
    pub progress_update: Option<
        crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate,
    >,
}

pub mod reference;
pub use reference::PublishedWorkflowProposalRef;
pub mod read_only;
pub use read_only::observe as observe_read_only_proposal;
/// A workflow proposal that landed: the proposal, the transition it opened, its commit
/// receipt from this actual publication.
#[derive(Debug)]
pub struct PerformedWorkflowProposal {
    pub proposal: PublishedWorkflowProposalRef,
    pub transition: EntityId,
    pub receipt: super::super::WorthQueryApplicationCommitReceipt,
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
}

/// What executing a workflow proposal produced. `ProjectionDenied` means the commit landed
/// but its receipt did not record this proposal on the instance's branch.
#[derive(Debug)]
pub enum WorkflowProposalOutcome {
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    Published(PerformedWorkflowProposal),
    /// The commit did not land. A fresh landed commit is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}


struct ProjectionContext {
    pub transition_identity_locator: AspectFieldLocator,
    pub proposal_identity_locator: AspectFieldLocator,
    pub proposal_node_path_locator: AspectFieldLocator,
    pub instance: PublishedWorkflowInstanceRef,
    pub operation: String,
    pub input_type: String,
    pub input_identity: [u8; 32],
    pub source_identity: Option<[u8; 32]>,
}

fn project(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    context: ProjectionContext,
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
        identity: identity.into(),
        operation: context.operation.into(),
        input_type: context.input_type.into(),
        input_identity: context.input_identity,
        source_identity: context.source_identity,
        node_path: node_path.into(),
    };
    WorkflowProposalOutcome::Published(PerformedWorkflowProposal {
        proposal: published,
        transition,
        receipt,
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
                    [AspectValue::String(InternedString::Stored(value))] => {
                        let value = value.as_string();
                        {
                            Some((entity, value.clone()))
                        }
                    }
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
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            receipt
                .committed_changes()
                .committed_field_values(entity, &[first, second])
                .and_then(|values| match values.as_slice() {
                    [a, b] => Some((entity, a.raw_text()?.to_owned(), b.raw_text()?.to_owned())),
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate.clone())
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowProposal<Schema, Operation, Input, Scope> {
    /// Controls retained as facts; each host call obtains fresh execution admission.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.request_scope()
    }
}
